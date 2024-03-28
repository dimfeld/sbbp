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
use schemars::JsonSchema;
use serde::Deserialize;
use tracing::{event, Level};
use url::Url;

use super::{
    queries, types::*, VideoId, CREATE_PERMISSION, OWNER_PERMISSION, READ_PERMISSION,
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
    Path(id): Path<VideoId>,
) -> Result<impl IntoResponse, Error> {
    let object = queries::get(&state.db, &auth, id).await?;

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
    FormOrJson(payload): FormOrJson<VideoCreatePayload>,
) -> Result<impl IntoResponse, Error> {
    let mut tx = state.db.begin().await.change_context(Error::Db)?;
    let result = queries::create(&mut *tx, &auth, payload).await?;
    tx.commit().await.change_context(Error::Db)?;

    Ok((StatusCode::CREATED, Json(result)))
}

async fn update(
    State(state): State<ServerState>,
    auth: Authed,
    Path(id): Path<VideoId>,
    FormOrJson(payload): FormOrJson<VideoUpdatePayload>,
) -> Result<impl IntoResponse, Error> {
    let mut tx = state.db.begin().await.change_context(Error::Db)?;

    let result = queries::update(&mut *tx, &auth, id, payload).await?;

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
    Path(id): Path<VideoId>,
) -> Result<impl IntoResponse, Error> {
    let mut tx = state.db.begin().await.change_context(Error::Db)?;

    let deleted = queries::delete(&mut *tx, &auth, id).await?;

    if !deleted {
        return Ok(StatusCode::NOT_FOUND);
    }

    tx.commit().await.change_context(Error::Db)?;

    Ok(StatusCode::OK)
}

#[derive(Deserialize, Debug, JsonSchema)]
struct CreateViaUrlPayload {
    url: Url,
}

async fn create_via_url(
    State(state): State<ServerState>,
    auth: Authed,
    FormOrJson(payload): FormOrJson<CreateViaUrlPayload>,
) -> Result<impl IntoResponse, Error> {
    let id = VideoId::new();
    sqlx::query!(
        "INSERT INTO videos (id, organization_id, processing_state, url, metadata) VALUES
        ($1, $2, $3, $4, '{}'::jsonb)",
        id.as_uuid(),
        auth.organization_id.as_uuid(),
        VideoProcessingState::Processing as _,
        payload.url.as_str()
    )
    .execute(&state.db)
    .await
    .change_context(Error::Db)?;

    crate::jobs::download::enqueue(
        &state,
        id,
        &crate::jobs::download::DownloadJobPayload {
            id,
            download_url: payload.url.to_string(),
            storage_prefix: id.to_string(),
        },
    )
    .await
    .change_context(Error::TaskQueue)
    .attach_printable("Failed to enqueue download job")?;

    Ok(())
}

pub fn create_routes() -> axum::Router<ServerState> {
    axum::Router::new()
        .route(
            "/download_video",
            routing::post(create_via_url)
                .route_layer(has_any_permission(vec![OWNER_PERMISSION, "org_admin"])),
        )
        .route(
            "/videos",
            routing::get(list).route_layer(has_any_permission(vec![READ_PERMISSION, "org_admin"])),
        )
        .route(
            "/videos/:id",
            routing::get(get).route_layer(has_any_permission(vec![READ_PERMISSION, "org_admin"])),
        )
        .route(
            "/videos/:id",
            routing::put(update).route_layer(has_any_permission(vec![
                WRITE_PERMISSION,
                OWNER_PERMISSION,
                "org_admin",
            ])),
        )
        .route(
            "/videos/:id",
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
    ) -> Vec<(VideoCreatePayload, VideoCreateResult)> {
        let mut tx = db.begin().await.unwrap();
        let mut objects = Vec::with_capacity(count);
        for i in 0..count {
            let id = VideoId::new();
            event!(Level::INFO, %id, "Creating test object {}", i);
            let payload = make_create_payload(i);
            let result = super::queries::create_raw(&mut *tx, id, organization_id, payload.clone())
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
            .get("videos")
            .send()
            .await
            .unwrap()
            .log_error()
            .await
            .unwrap()
            .json::<Vec<serde_json::Value>>()
            .await
            .unwrap();

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
                result["processing_state"],
                serde_json::to_value(&added.processing_state).unwrap(),
                "field processing_state"
            );
            assert_eq!(
                result["url"],
                serde_json::to_value(&added.url).unwrap(),
                "field url"
            );
            assert_eq!(
                result["title"],
                serde_json::to_value(&added.title).unwrap(),
                "field title"
            );

            assert_eq!(payload.title, added.title, "create result field title");
            assert_eq!(
                result["duration"],
                serde_json::to_value(&added.duration).unwrap(),
                "field duration"
            );
            assert_eq!(
                result["author"],
                serde_json::to_value(&added.author).unwrap(),
                "field author"
            );
            assert_eq!(
                result["date"],
                serde_json::to_value(&added.date).unwrap(),
                "field date"
            );
            assert_eq!(
                result["metadata"],
                serde_json::to_value(&added.metadata).unwrap(),
                "field metadata"
            );
            assert_eq!(
                result["read"],
                serde_json::to_value(&added.read).unwrap(),
                "field read"
            );

            assert_eq!(payload.read, added.read, "create result field read");
            assert_eq!(
                result["progress"],
                serde_json::to_value(&added.progress).unwrap(),
                "field progress"
            );

            assert_eq!(
                payload.progress, added.progress,
                "create result field progress"
            );
            assert_eq!(
                result["images"],
                serde_json::to_value(&added.images).unwrap(),
                "field images"
            );
            assert_eq!(
                result["transcript"],
                serde_json::to_value(&added.transcript).unwrap(),
                "field transcript"
            );
            assert_eq!(
                result["summary"],
                serde_json::to_value(&added.summary).unwrap(),
                "field summary"
            );
            assert_eq!(
                result["processed_path"],
                serde_json::to_value(&added.processed_path).unwrap(),
                "field processed_path"
            );

            assert_eq!(result["_permission"], "owner");
        }

        let results = user
            .client
            .get("videos")
            .send()
            .await
            .unwrap()
            .log_error()
            .await
            .unwrap()
            .json::<Vec<serde_json::Value>>()
            .await
            .unwrap();

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
                result["processing_state"],
                serde_json::to_value(&added.processing_state).unwrap(),
                "list result field processing_state"
            );
            assert_eq!(
                result["url"],
                serde_json::to_value(&added.url).unwrap(),
                "list result field url"
            );
            assert_eq!(
                result["title"],
                serde_json::to_value(&added.title).unwrap(),
                "list result field title"
            );
            assert_eq!(
                result["duration"],
                serde_json::to_value(&added.duration).unwrap(),
                "list result field duration"
            );
            assert_eq!(
                result["author"],
                serde_json::to_value(&added.author).unwrap(),
                "list result field author"
            );
            assert_eq!(
                result["date"],
                serde_json::to_value(&added.date).unwrap(),
                "list result field date"
            );
            assert_eq!(
                result["metadata"],
                serde_json::to_value(&added.metadata).unwrap(),
                "list result field metadata"
            );
            assert_eq!(
                result["read"],
                serde_json::to_value(&added.read).unwrap(),
                "list result field read"
            );
            assert_eq!(
                result["progress"],
                serde_json::to_value(&added.progress).unwrap(),
                "list result field progress"
            );
            assert_eq!(
                result["images"],
                serde_json::to_value(&added.images).unwrap(),
                "list result field images"
            );
            assert_eq!(
                result["transcript"],
                serde_json::to_value(&added.transcript).unwrap(),
                "list result field transcript"
            );
            assert_eq!(
                result["summary"],
                serde_json::to_value(&added.summary).unwrap(),
                "list result field summary"
            );
            assert_eq!(
                result["processed_path"],
                serde_json::to_value(&added.processed_path).unwrap(),
                "list result field processed_path"
            );
            assert_eq!(result["_permission"], "write");
        }

        let response = no_roles_user.client.get("videos").send().await.unwrap();

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
            .get("videos")
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
            .get(&format!("videos/{}", added_objects[1].1.id))
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
            result["processing_state"],
            serde_json::to_value(&added.processing_state).unwrap(),
            "get result field processing_state"
        );
        assert_eq!(
            result["url"],
            serde_json::to_value(&added.url).unwrap(),
            "get result field url"
        );
        assert_eq!(
            result["title"],
            serde_json::to_value(&added.title).unwrap(),
            "get result field title"
        );

        assert_eq!(added.title, payload.title, "create result field title");
        assert_eq!(
            result["duration"],
            serde_json::to_value(&added.duration).unwrap(),
            "get result field duration"
        );
        assert_eq!(
            result["author"],
            serde_json::to_value(&added.author).unwrap(),
            "get result field author"
        );
        assert_eq!(
            result["date"],
            serde_json::to_value(&added.date).unwrap(),
            "get result field date"
        );
        assert_eq!(
            result["metadata"],
            serde_json::to_value(&added.metadata).unwrap(),
            "get result field metadata"
        );
        assert_eq!(
            result["read"],
            serde_json::to_value(&added.read).unwrap(),
            "get result field read"
        );

        assert_eq!(added.read, payload.read, "create result field read");
        assert_eq!(
            result["progress"],
            serde_json::to_value(&added.progress).unwrap(),
            "get result field progress"
        );

        assert_eq!(
            added.progress, payload.progress,
            "create result field progress"
        );
        assert_eq!(
            result["images"],
            serde_json::to_value(&added.images).unwrap(),
            "get result field images"
        );
        assert_eq!(
            result["transcript"],
            serde_json::to_value(&added.transcript).unwrap(),
            "get result field transcript"
        );
        assert_eq!(
            result["summary"],
            serde_json::to_value(&added.summary).unwrap(),
            "get result field summary"
        );
        assert_eq!(
            result["processed_path"],
            serde_json::to_value(&added.processed_path).unwrap(),
            "get result field processed_path"
        );

        assert_eq!(result["_permission"], "owner");

        let result = user
            .client
            .get(&format!("videos/{}", added_objects[1].1.id))
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
            result["processing_state"],
            serde_json::to_value(&added.processing_state).unwrap(),
            "get result field processing_state"
        );
        assert_eq!(
            result["url"],
            serde_json::to_value(&added.url).unwrap(),
            "get result field url"
        );
        assert_eq!(
            result["title"],
            serde_json::to_value(&added.title).unwrap(),
            "get result field title"
        );
        assert_eq!(
            result["duration"],
            serde_json::to_value(&added.duration).unwrap(),
            "get result field duration"
        );
        assert_eq!(
            result["author"],
            serde_json::to_value(&added.author).unwrap(),
            "get result field author"
        );
        assert_eq!(
            result["date"],
            serde_json::to_value(&added.date).unwrap(),
            "get result field date"
        );
        assert_eq!(
            result["metadata"],
            serde_json::to_value(&added.metadata).unwrap(),
            "get result field metadata"
        );
        assert_eq!(
            result["read"],
            serde_json::to_value(&added.read).unwrap(),
            "get result field read"
        );
        assert_eq!(
            result["progress"],
            serde_json::to_value(&added.progress).unwrap(),
            "get result field progress"
        );
        assert_eq!(
            result["images"],
            serde_json::to_value(&added.images).unwrap(),
            "get result field images"
        );
        assert_eq!(
            result["transcript"],
            serde_json::to_value(&added.transcript).unwrap(),
            "get result field transcript"
        );
        assert_eq!(
            result["summary"],
            serde_json::to_value(&added.summary).unwrap(),
            "get result field summary"
        );
        assert_eq!(
            result["processed_path"],
            serde_json::to_value(&added.processed_path).unwrap(),
            "get result field processed_path"
        );
        assert_eq!(result["_permission"], "write");

        let response = no_roles_user
            .client
            .get(&format!("videos/{}", added_objects[1].1.id))
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
            .put(&format!("videos/{}", added_objects[1].1.id))
            .json(&update_payload)
            .send()
            .await
            .unwrap()
            .log_error()
            .await
            .unwrap();

        let updated: serde_json::Value = admin_user
            .client
            .get(&format!("videos/{}", added_objects[1].1.id))
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
            updated["title"],
            serde_json::to_value(&update_payload.title).unwrap(),
            "field title"
        );
        assert_eq!(
            updated["read"],
            serde_json::to_value(&update_payload.read).unwrap(),
            "field read"
        );
        assert_eq!(
            updated["progress"],
            serde_json::to_value(&update_payload.progress).unwrap(),
            "field progress"
        );
        assert_eq!(updated["_permission"], "owner");

        // TODO Test that owner can not write fields which are not writable by anyone.
        // TODO Test that user can not update fields which are writable by owner but not user

        // Make sure that no other objects were updated
        let non_updated: serde_json::Value = admin_user
            .client
            .get(&format!("videos/{}", added_objects[0].1.id))
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
            non_updated["processing_state"],
            serde_json::to_value(&added_objects[0].1.processing_state).unwrap(),
            "field processing_state"
        );
        assert_eq!(
            non_updated["url"],
            serde_json::to_value(&added_objects[0].1.url).unwrap(),
            "field url"
        );
        assert_eq!(
            non_updated["title"],
            serde_json::to_value(&added_objects[0].1.title).unwrap(),
            "field title"
        );
        assert_eq!(
            non_updated["duration"],
            serde_json::to_value(&added_objects[0].1.duration).unwrap(),
            "field duration"
        );
        assert_eq!(
            non_updated["author"],
            serde_json::to_value(&added_objects[0].1.author).unwrap(),
            "field author"
        );
        assert_eq!(
            non_updated["date"],
            serde_json::to_value(&added_objects[0].1.date).unwrap(),
            "field date"
        );
        assert_eq!(
            non_updated["metadata"],
            serde_json::to_value(&added_objects[0].1.metadata).unwrap(),
            "field metadata"
        );
        assert_eq!(
            non_updated["read"],
            serde_json::to_value(&added_objects[0].1.read).unwrap(),
            "field read"
        );
        assert_eq!(
            non_updated["progress"],
            serde_json::to_value(&added_objects[0].1.progress).unwrap(),
            "field progress"
        );
        assert_eq!(
            non_updated["images"],
            serde_json::to_value(&added_objects[0].1.images).unwrap(),
            "field images"
        );
        assert_eq!(
            non_updated["transcript"],
            serde_json::to_value(&added_objects[0].1.transcript).unwrap(),
            "field transcript"
        );
        assert_eq!(
            non_updated["summary"],
            serde_json::to_value(&added_objects[0].1.summary).unwrap(),
            "field summary"
        );
        assert_eq!(
            non_updated["processed_path"],
            serde_json::to_value(&added_objects[0].1.processed_path).unwrap(),
            "field processed_path"
        );
        assert_eq!(non_updated["_permission"], "owner");

        let response = no_roles_user
            .client
            .put(&format!("videos/{}", added_objects[1].1.id))
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
            .delete(&format!("videos/{}", added_objects[1].1.id))
            .send()
            .await
            .unwrap()
            .log_error()
            .await
            .unwrap();

        let response = admin_user
            .client
            .get(&format!("videos/{}", added_objects[1].1.id))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);

        // Delete should not happen without permissions
        let response = no_roles_user
            .client
            .delete(&format!("videos/{}", added_objects[0].1.id))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), reqwest::StatusCode::FORBIDDEN);

        // Make sure other objects still exist
        let response = admin_user
            .client
            .get(&format!("videos/{}", added_objects[0].1.id))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), reqwest::StatusCode::OK);
    }
}