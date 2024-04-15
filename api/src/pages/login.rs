use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    routing,
};
use axum_extra::extract::Query;
use axum_htmx::{HxLocation, HxPushUrl, HxRetarget, HxTarget, HxTrigger};
use error_stack::ResultExt;
use filigree::{
    auth::password::{login_with_password, EmailAndPassword},
    errors::HttpError,
    extract::{FormOrJson, ValidatedForm},
};
use maud::{html, Markup};
use schemars::JsonSchema;

use crate::{
    auth::{has_any_permission, Authed},
    pages::{error::HtmlError, layout::root_layout_page},
    server::ServerState,
    Error,
};

#[derive(serde::Deserialize, Debug)]
struct RedirectTo {
    redirect_to: Option<String>,
}

#[derive(serde::Deserialize, Debug, JsonSchema)]
struct LoginForm {
    #[validate(email)]
    email: String,
    password: Option<String>,
    redirect_to: Option<String>,
}

async fn login_form(
    State(state): State<ServerState>,
    HxTrigger(trigger): HxTrigger,
    cookies: tower_cookies::Cookies,
    mut form: ValidatedForm<LoginForm>,
) -> Result<Response, HtmlError> {
    if let Some(data) = &form.data {
        let trigger = trigger.as_deref().unwrap_or_default();
        let password = data.password.as_deref().unwrap_or_default();
        if trigger == "passwordless" || (trigger != "login" && password.is_empty()) {
            let result = filigree::auth::passwordless_email_login::setup_passwordless_login(
                &state,
                data.email.clone(),
            )
            .await;

            let message = if result.is_err() {
                "An error occurred. Please try again later."
            } else {
                "Check your email for a link to log in."
            };

            let redirect_to = data.redirect_to.clone();
            return Ok(login_page_form(form, message, redirect_to).into_response());
        } else {
            if password.is_empty() {
                form.errors.add_field("password", "Password is required");
            }

            let login_result = filigree::auth::password::login_with_password(
                &state.session_backend,
                &cookies,
                EmailAndPassword {
                    email: data.email.clone(),
                    password: password.to_string(),
                },
            )
            .await;

            if let Err(e) = login_result {
                let message = match e.status_code() {
                    StatusCode::UNAUTHORIZED => "Incorrect email or password",
                    _ => "An error occurred. Please try again later.",
                };

                let redirect_to = data.redirect_to.clone();
                return Ok(login_page_form(form, message, redirect_to).into_response());
            }

            let redirect_to = data
                .redirect_to
                .as_deref()
                // Don't redirect if it points to some other website
                .filter(|r| r.starts_with('/'))
                .unwrap_or("/");
            let redirect_to = redirect_to.parse().unwrap_or_else(|_| "/".parse().unwrap());

            return Ok((HxLocation::from_uri(redirect_to), StatusCode::OK).into_response());
        }
    }

    Ok(login_page_form(form, "", None).into_response())
}

fn login_page_form(
    ValidatedForm { form, errors, .. }: ValidatedForm<LoginForm>,
    message: &str,
    redirect_to: Option<String>,
) -> Markup {
    let redirect_to = redirect_to
        .as_deref()
        .or_else(|| form["redirect_to"].as_str().clone());

    html! {
        @if !message.is_empty() {
            div .text-red-50 { (message) }
        }

        input type="hidden" name="redirect_to" value=[redirect_to];
        div.flex.flex-col.gap-2 {
            label .text-red-200 for="email" { "Email" }
            input #email .input.input-bordered required type="email" name="email" value=[form["email"].as_str()];
            span .text-red-50 { (errors.field("email")) }
        }
        div.flex.flex-col.gap-2 {
            label for="password" { "Password" }
            input #password .input.input-bordered type="password" name="password" autocomplete="off";
            span .text-red-50 { (errors.field("password")) }
        }
        div.flex.gap-4 {
            button #login .btn.btn-primary { "Login" }
            button #passwordless .btn { "Login through email"}
        }
    }
}

async fn login_page(Query(query): Query<RedirectTo>) -> impl IntoResponse {
    let body = html! {
        form .container.mx-auto.flex.flex-col.gap-4.p-4.max-w-lg hx-post="/login" {
            (login_page_form(Default::default(), "", query.redirect_to))
        }
    };

    root_layout_page(None, "Login", body)
}

pub fn create_routes() -> axum::Router<ServerState> {
    axum::Router::new()
        .route("/login", routing::get(login_page))
        .route("/login", routing::post(login_form))
}
