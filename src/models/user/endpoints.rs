#![allow(unused_imports, unused_variables, dead_code)]
use std::{borrow::Cow, str::FromStr};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing,
};
use axum_extra::extract::Query;
use axum_jsonschema::Json;
use error_stack::ResultExt;
use filigree::{
    auth::{AuthError, ObjectPermission},
    extract::FormOrJson,
};
use tracing::{event, Level};

use super::{
    queries, types::*, UserId, CREATE_PERMISSION, OWNER_PERMISSION, READ_PERMISSION,
    WRITE_PERMISSION,
};
use crate::{
    auth::{has_any_permission, Authed},
    server::ServerState,
    Error,
};

async fn get(
    State(state): State<ServerState>,
    auth: Authed,
    Path(id): Path<UserId>,
) -> Result<impl IntoResponse, Error> {
    let object = queries::get(&state.db, &auth, &id).await?;

    Ok(Json(object))
}

async fn list(
    State(state): State<ServerState>,
    auth: Authed,
    Query(qs): Query<queries::ListQueryFilters>,
) -> Result<impl IntoResponse, Error> {
    let results = queries::list(&state.db, &auth, &qs).await?;

    Ok(Json(results))
}

async fn create(
    State(state): State<ServerState>,
    auth: Authed,
    FormOrJson(payload): FormOrJson<UserCreatePayload>,
) -> Result<impl IntoResponse, Error> {
    let mut tx = state.db.begin().await.change_context(Error::Db)?;
    let result = queries::create(&mut *tx, &auth, payload).await?;
    tx.commit().await.change_context(Error::Db)?;

    Ok((StatusCode::CREATED, Json(result)))
}

async fn update(
    State(state): State<ServerState>,
    auth: Authed,
    Path(id): Path<UserId>,
    FormOrJson(payload): FormOrJson<UserUpdatePayload>,
) -> Result<impl IntoResponse, Error> {
    let mut tx = state.db.begin().await.change_context(Error::Db)?;

    let result = queries::update(&mut *tx, &auth, &id, payload).await?;

    tx.commit().await.change_context(Error::Db)?;

    if result {
        Ok(StatusCode::OK)
    } else {
        Ok(StatusCode::NOT_FOUND)
    }
}

async fn delete(
    State(state): State<ServerState>,
    auth: Authed,
    Path(id): Path<UserId>,
) -> Result<impl IntoResponse, Error> {
    let mut tx = state.db.begin().await.change_context(Error::Db)?;

    let deleted = queries::delete(&mut *tx, &auth, &id).await?;

    if !deleted {
        return Ok(StatusCode::NOT_FOUND);
    }

    tx.commit().await.change_context(Error::Db)?;

    Ok(StatusCode::OK)
}

pub fn create_routes() -> axum::Router<ServerState> {
    axum::Router::new()
        .route(
            "/users",
            routing::get(list).route_layer(has_any_permission(vec![READ_PERMISSION, "org_admin"])),
        )
        .route(
            "/users/:id",
            routing::get(get).route_layer(has_any_permission(vec![READ_PERMISSION, "org_admin"])),
        )
        .route(
            "/users/:id",
            routing::put(update).route_layer(has_any_permission(vec![
                WRITE_PERMISSION,
                OWNER_PERMISSION,
                "org_admin",
            ])),
        )
        .route(
            "/users/:id",
            routing::delete(delete)
                .route_layer(has_any_permission(vec![CREATE_PERMISSION, "org_admin"])),
        )
}

#[cfg(test)]
mod test {
    use filigree::testing::ResponseExt;
    use futures::{StreamExt, TryStreamExt};
    use tracing::{event, Level};

    use super::{
        super::testing::{make_create_payload, make_update_payload},
        *,
    };
    use crate::{
        models::organization::OrganizationId,
        tests::{start_app, BootstrappedData},
    };

    async fn setup_test_objects(
        db: &sqlx::PgPool,
        organization_id: OrganizationId,
        count: usize,
    ) -> Vec<(UserCreatePayload, UserCreateResult)> {
        let mut tx = db.begin().await.unwrap();
        let mut objects = Vec::with_capacity(count);
        for i in 0..count {
            let id = UserId::new();
            event!(Level::INFO, %id, "Creating test object {}", i);
            let payload = make_create_payload(i);
            let result =
                super::queries::create_raw(&mut *tx, &id, &organization_id, payload.clone())
                    .await
                    .expect("Creating test object failed");

            objects.push((payload, result));
        }

        tx.commit().await.unwrap();
        objects
    }

    #[sqlx::test]
    async fn list_objects(pool: sqlx::PgPool) {
        let (
            _app,
            BootstrappedData {
                organization,
                admin_user,
                no_roles_user,
                user,
                ..
            },
        ) = start_app(pool.clone()).await;

        let added_objects = setup_test_objects(&pool, organization.id, 3).await;

        let results = admin_user
            .client
            .get("users")
            .send()
            .await
            .unwrap()
            .log_error()
            .await
            .unwrap()
            .json::<Vec<serde_json::Value>>()
            .await
            .unwrap();

        let fixed_users = [
            admin_user.user_id.to_string(),
            user.user_id.to_string(),
            no_roles_user.user_id.to_string(),
        ];
        let results = results
            .into_iter()
            .filter(|value| {
                !fixed_users
                    .iter()
                    .any(|i| i == value["id"].as_str().unwrap())
            })
            .collect::<Vec<_>>();

        assert_eq!(results.len(), added_objects.len());

        for result in results {
            let (payload, added) = added_objects
                .iter()
                .find(|i| i.1.id.to_string() == result["id"].as_str().unwrap())
                .expect("Returned object did not match any of the added objects");
            assert_eq!(
                result["id"],
                serde_json::to_value(&added.id).unwrap(),
                "field id"
            );
            assert_eq!(
                result["organization_id"],
                serde_json::to_value(&added.organization_id).unwrap(),
                "field organization_id"
            );
            assert_eq!(
                result["updated_at"],
                serde_json::to_value(&added.updated_at).unwrap(),
                "field updated_at"
            );
            assert_eq!(
                result["created_at"],
                serde_json::to_value(&added.created_at).unwrap(),
                "field created_at"
            );
            assert_eq!(
                result["name"],
                serde_json::to_value(&added.name).unwrap(),
                "field name"
            );

            assert_eq!(payload.name, added.name, "create result field name");
            assert_eq!(
                result["email"],
                serde_json::to_value(&added.email).unwrap(),
                "field email"
            );

            assert_eq!(payload.email, added.email, "create result field email");
            assert_eq!(
                result["avatar_url"],
                serde_json::to_value(&added.avatar_url).unwrap(),
                "field avatar_url"
            );

            assert_eq!(
                payload.avatar_url, added.avatar_url,
                "create result field avatar_url"
            );

            assert_eq!(result["_permission"], "owner");

            assert_eq!(
                result.get("password_hash"),
                None,
                "field password_hash should be omitted"
            );
        }

        let results = user
            .client
            .get("users")
            .send()
            .await
            .unwrap()
            .log_error()
            .await
            .unwrap()
            .json::<Vec<serde_json::Value>>()
            .await
            .unwrap();

        let fixed_users = [
            admin_user.user_id.to_string(),
            user.user_id.to_string(),
            no_roles_user.user_id.to_string(),
        ];
        let results = results
            .into_iter()
            .filter(|value| {
                !fixed_users
                    .iter()
                    .any(|i| i == value["id"].as_str().unwrap())
            })
            .collect::<Vec<_>>();

        for result in results {
            let (payload, added) = added_objects
                .iter()
                .find(|i| i.1.id.to_string() == result["id"].as_str().unwrap())
                .expect("Returned object did not match any of the added objects");
            assert_eq!(
                result["id"],
                serde_json::to_value(&added.id).unwrap(),
                "list result field id"
            );
            assert_eq!(
                result["organization_id"],
                serde_json::to_value(&added.organization_id).unwrap(),
                "list result field organization_id"
            );
            assert_eq!(
                result["updated_at"],
                serde_json::to_value(&added.updated_at).unwrap(),
                "list result field updated_at"
            );
            assert_eq!(
                result["created_at"],
                serde_json::to_value(&added.created_at).unwrap(),
                "list result field created_at"
            );
            assert_eq!(
                result["name"],
                serde_json::to_value(&added.name).unwrap(),
                "list result field name"
            );
            assert_eq!(
                result["email"],
                serde_json::to_value(&added.email).unwrap(),
                "list result field email"
            );
            assert_eq!(
                result["avatar_url"],
                serde_json::to_value(&added.avatar_url).unwrap(),
                "list result field avatar_url"
            );
            assert_eq!(result["_permission"], "write");

            assert_eq!(
                result.get("password_hash"),
                None,
                "field password_hash should be omitted"
            );
        }

        let response = no_roles_user.client.get("users").send().await.unwrap();

        assert_eq!(response.status(), reqwest::StatusCode::FORBIDDEN);
    }

    #[sqlx::test]
    async fn list_fetch_specific_ids(pool: sqlx::PgPool) {
        let (
            _app,
            BootstrappedData {
                organization, user, ..
            },
        ) = start_app(pool.clone()).await;

        let added_objects = setup_test_objects(&pool, organization.id, 3).await;

        let results = user
            .client
            .get("users")
            .query(&[("id", added_objects[0].1.id), ("id", added_objects[2].1.id)])
            .send()
            .await
            .unwrap()
            .log_error()
            .await
            .unwrap()
            .json::<Vec<serde_json::Value>>()
            .await
            .unwrap();

        assert_eq!(results.len(), 2);
        assert!(results
            .iter()
            .any(|o| o["id"] == added_objects[0].1.id.to_string()));
        assert!(results
            .iter()
            .any(|o| o["id"] == added_objects[2].1.id.to_string()));
    }

    #[sqlx::test]
    #[ignore = "todo"]
    async fn list_order_by(_pool: sqlx::PgPool) {}

    #[sqlx::test]
    #[ignore = "todo"]
    async fn list_paginated(_pool: sqlx::PgPool) {}

    #[sqlx::test]
    #[ignore = "todo"]
    async fn list_filters(_pool: sqlx::PgPool) {}

    #[sqlx::test]
    async fn get_object(pool: sqlx::PgPool) {
        let (
            _app,
            BootstrappedData {
                organization,
                admin_user,
                user,
                no_roles_user,
                ..
            },
        ) = start_app(pool.clone()).await;

        let added_objects = setup_test_objects(&pool, organization.id, 2).await;

        let result = admin_user
            .client
            .get(&format!("users/{}", added_objects[1].1.id))
            .send()
            .await
            .unwrap()
            .log_error()
            .await
            .unwrap()
            .json::<serde_json::Value>()
            .await
            .unwrap();

        let (payload, added) = &added_objects[1];
        assert_eq!(
            result["id"],
            serde_json::to_value(&added.id).unwrap(),
            "get result field id"
        );
        assert_eq!(
            result["organization_id"],
            serde_json::to_value(&added.organization_id).unwrap(),
            "get result field organization_id"
        );
        assert_eq!(
            result["updated_at"],
            serde_json::to_value(&added.updated_at).unwrap(),
            "get result field updated_at"
        );
        assert_eq!(
            result["created_at"],
            serde_json::to_value(&added.created_at).unwrap(),
            "get result field created_at"
        );
        assert_eq!(
            result["name"],
            serde_json::to_value(&added.name).unwrap(),
            "get result field name"
        );

        assert_eq!(added.name, payload.name, "create result field name");
        assert_eq!(
            result["email"],
            serde_json::to_value(&added.email).unwrap(),
            "get result field email"
        );

        assert_eq!(added.email, payload.email, "create result field email");
        assert_eq!(
            result["avatar_url"],
            serde_json::to_value(&added.avatar_url).unwrap(),
            "get result field avatar_url"
        );

        assert_eq!(
            added.avatar_url, payload.avatar_url,
            "create result field avatar_url"
        );

        assert_eq!(result["_permission"], "owner");

        assert_eq!(
            result.get("password_hash"),
            None,
            "field password_hash should be omitted"
        );

        let result = user
            .client
            .get(&format!("users/{}", added_objects[1].1.id))
            .send()
            .await
            .unwrap()
            .log_error()
            .await
            .unwrap()
            .json::<serde_json::Value>()
            .await
            .unwrap();

        let (payload, added) = &added_objects[1];
        assert_eq!(
            result["id"],
            serde_json::to_value(&added.id).unwrap(),
            "get result field id"
        );
        assert_eq!(
            result["organization_id"],
            serde_json::to_value(&added.organization_id).unwrap(),
            "get result field organization_id"
        );
        assert_eq!(
            result["updated_at"],
            serde_json::to_value(&added.updated_at).unwrap(),
            "get result field updated_at"
        );
        assert_eq!(
            result["created_at"],
            serde_json::to_value(&added.created_at).unwrap(),
            "get result field created_at"
        );
        assert_eq!(
            result["name"],
            serde_json::to_value(&added.name).unwrap(),
            "get result field name"
        );
        assert_eq!(
            result["email"],
            serde_json::to_value(&added.email).unwrap(),
            "get result field email"
        );
        assert_eq!(
            result["avatar_url"],
            serde_json::to_value(&added.avatar_url).unwrap(),
            "get result field avatar_url"
        );
        assert_eq!(result["_permission"], "write");

        assert_eq!(
            result.get("password_hash"),
            None,
            "field password_hash should be omitted"
        );

        let response = no_roles_user
            .client
            .get(&format!("users/{}", added_objects[1].1.id))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), reqwest::StatusCode::FORBIDDEN);
    }

    #[sqlx::test]
    async fn update_object(pool: sqlx::PgPool) {
        let (
            _app,
            BootstrappedData {
                organization,
                admin_user,
                no_roles_user,
                ..
            },
        ) = start_app(pool.clone()).await;

        let added_objects = setup_test_objects(&pool, organization.id, 2).await;

        let update_payload = make_update_payload(20);
        admin_user
            .client
            .put(&format!("users/{}", added_objects[1].1.id))
            .json(&update_payload)
            .send()
            .await
            .unwrap()
            .log_error()
            .await
            .unwrap();

        let updated: serde_json::Value = admin_user
            .client
            .get(&format!("users/{}", added_objects[1].1.id))
            .send()
            .await
            .unwrap()
            .log_error()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        assert_eq!(
            updated["name"],
            serde_json::to_value(&update_payload.name).unwrap(),
            "field name"
        );
        assert_eq!(
            updated["email"],
            serde_json::to_value(&update_payload.email).unwrap(),
            "field email"
        );
        assert_eq!(
            updated["avatar_url"],
            serde_json::to_value(&update_payload.avatar_url).unwrap(),
            "field avatar_url"
        );
        assert_eq!(updated["_permission"], "owner");

        // TODO Test that owner can not write fields which are not writable by anyone.
        // TODO Test that user can not update fields which are writable by owner but not user

        // Make sure that no other objects were updated
        let non_updated: serde_json::Value = admin_user
            .client
            .get(&format!("users/{}", added_objects[0].1.id))
            .send()
            .await
            .unwrap()
            .log_error()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        assert_eq!(
            non_updated["id"],
            serde_json::to_value(&added_objects[0].1.id).unwrap(),
            "field id"
        );
        assert_eq!(
            non_updated["organization_id"],
            serde_json::to_value(&added_objects[0].1.organization_id).unwrap(),
            "field organization_id"
        );
        assert_eq!(
            non_updated["updated_at"],
            serde_json::to_value(&added_objects[0].1.updated_at).unwrap(),
            "field updated_at"
        );
        assert_eq!(
            non_updated["created_at"],
            serde_json::to_value(&added_objects[0].1.created_at).unwrap(),
            "field created_at"
        );
        assert_eq!(
            non_updated["name"],
            serde_json::to_value(&added_objects[0].1.name).unwrap(),
            "field name"
        );
        assert_eq!(
            non_updated["email"],
            serde_json::to_value(&added_objects[0].1.email).unwrap(),
            "field email"
        );
        assert_eq!(
            non_updated["avatar_url"],
            serde_json::to_value(&added_objects[0].1.avatar_url).unwrap(),
            "field avatar_url"
        );
        assert_eq!(non_updated["_permission"], "owner");

        let response = no_roles_user
            .client
            .put(&format!("users/{}", added_objects[1].1.id))
            .json(&update_payload)
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), reqwest::StatusCode::FORBIDDEN);
    }

    #[sqlx::test]
    async fn delete_object(pool: sqlx::PgPool) {
        let (
            _app,
            BootstrappedData {
                organization,
                admin_user,
                no_roles_user,
                ..
            },
        ) = start_app(pool.clone()).await;

        let added_objects = setup_test_objects(&pool, organization.id, 2).await;

        admin_user
            .client
            .delete(&format!("users/{}", added_objects[1].1.id))
            .send()
            .await
            .unwrap()
            .log_error()
            .await
            .unwrap();

        let response = admin_user
            .client
            .get(&format!("users/{}", added_objects[1].1.id))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);

        // Delete should not happen without permissions
        let response = no_roles_user
            .client
            .delete(&format!("users/{}", added_objects[0].1.id))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), reqwest::StatusCode::FORBIDDEN);

        // Make sure other objects still exist
        let response = admin_user
            .client
            .get(&format!("users/{}", added_objects[0].1.id))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), reqwest::StatusCode::OK);
    }
}
