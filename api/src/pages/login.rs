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
    error_reporting::HandleErrorReport,
    errors::HttpError,
    extract::{FormOrJson, ValidatedForm},
    html::HtmlList,
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
) -> Response {
    let redirect_to = form.data.as_ref().and_then(|d| d.redirect_to.clone());

    let Some(data) = &form.data else {
        // form error
        return login_page_form(form, "", redirect_to).into_response();
    };

    let trigger = trigger.as_deref().unwrap_or_default();
    let password = data.password.as_deref().unwrap_or_default();
    let message = if trigger == "passwordless" || (trigger != "login" && password.is_empty()) {
        let result = filigree::auth::passwordless_email_login::setup_passwordless_login(
            &state,
            data.email.clone(),
        )
        .await
        .report_error();

        if result.is_err() {
            "An error occurred. Please try again later."
        } else {
            "Check your email for a link to log in."
        }
    } else if password.is_empty() {
        form.errors.add_field("password", "Password is required");
        ""
    } else {
        let login_result = filigree::auth::password::login_with_password(
            &state.session_backend,
            &cookies,
            EmailAndPassword {
                email: data.email.clone(),
                password: password.to_string(),
            },
        )
        .await
        .report_error();

        match login_result.status_code() {
            StatusCode::OK => {
                let redirect_to = redirect_to
                    .as_deref()
                    // Don't redirect if it points to some other website
                    .filter(|r| r.starts_with('/'))
                    .unwrap_or("/");
                let redirect_to = redirect_to.parse().unwrap_or_else(|_| "/".parse().unwrap());

                return (HxLocation::from_uri(redirect_to), StatusCode::OK).into_response();
            }
            StatusCode::UNAUTHORIZED => "Incorrect email or password",
            _ => "An error occurred. Please try again later.",
        }
    };

    login_page_form(form, message, redirect_to).into_response()
}

fn login_page_form(
    ValidatedForm { form, errors, .. }: ValidatedForm<LoginForm>,
    message: &str,
    redirect_to: Option<String>,
) -> Markup {
    let redirect_to = redirect_to
        .as_deref()
        .or_else(|| form["redirect_to"].as_str());

    html! {
        ul.text-red-50 {
            @if !message.is_empty() {
                li { (message) }
            }

            @if !errors.messages.is_empty() {
                (HtmlList::new(&errors.messages))
            }
        }

        input type="hidden" name="redirect_to" value=[redirect_to];
        div.flex.flex-col.gap-2 {
            label .text-red-200 for="email" { "Email" }
            input #email .input.input-bordered required type="email" name="email" value=[form["email"].as_str()];
            (errors.field_ul("email", "text-red-50"))
        }
        div.flex.flex-col.gap-2 {
            label for="password" { "Password" }
            input #password .input.input-bordered type="password" name="password" autocomplete="off";
            (errors.field_ul("password", "text-red-50"))
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
