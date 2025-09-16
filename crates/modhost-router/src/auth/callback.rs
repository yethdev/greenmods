//! The route for the GitHub authentication callback.

use axum::{
    body::Body,
    extract::State,
    http::{
        HeaderValue, StatusCode, Uri,
        header::{LOCATION, SET_COOKIE},
    },
    response::Response,
};
use axum_extra::extract::Host;
use modhost_core::Result;
use modhost_db::{create_token, prelude::Users, users};
use modhost_middleware::scheme::Scheme;
use modhost_server_core::{github::create_github_client, state::AppState};
use oauth2::{RedirectUrl, TokenResponse};
use sea_orm::{ActiveValue::Set, EntityTrait, sea_query::OnConflict};
use std::collections::HashMap;

use super::CALLBACK_URL;

/// GitHub Auth Callback
///
/// Complete the GitHub login flow.
#[utoipa::path(
    get,
    path = "/github/callback",
    tag = "Auth",
    responses(
        (status = 307, description = "Success, redirecting to user info."),
    ),
    params(
        ("code" = String, Query, description = "Response code from GitHub"),
        ("state" = String, Query, description = "Response state from GitHub"),
    ),
)]
pub async fn callback_handler(
    State(state): State<AppState>,
    Host(host): Host,
    Scheme(scheme): Scheme,
    url: Uri,
) -> Result<Response> {
    let query = url::form_urlencoded::parse(url.query().unwrap().as_bytes())
        .into_owned()
        .collect::<HashMap<String, String>>();

    let code = query.get("code").unwrap();
    let to = query.get("to");

    let auth_url = format!("{}://{}{}", scheme, host, CALLBACK_URL);

    let client = state.auth.set_redirect_uri(RedirectUrl::new(auth_url)?);

    match client
        .exchange_code(oauth2::AuthorizationCode::new(code.to_owned()))
        .request_async(&oauth2::reqwest::Client::new())
        .await
    {
        Ok(token) => {
            let github_token = token.access_token().secret();
            let client = create_github_client(github_token)?;
            let me = client.current().user().await?;

            let user = Users::insert(users::ActiveModel {
                username: Set(me.login),
                github_id: Set(me.id.0 as i32),
                admin: Set(false),
                moderator: Set(false),
                ..Default::default()
            })
            .on_conflict(
                OnConflict::column(users::Column::GithubId)
                    .update_column(users::Column::Username)
                    .to_owned(),
            )
            .exec_with_returning(&state.db)
            .await?;

            let token = create_token(user.id, &state.db).await?;

            let cookie_value = format!(
                "auth-token={}; HttpOnly; Path=/; Domain={}",
                token.value,
                sanitize_port(&host)
            );

            let mut response = Response::builder()
                .status(StatusCode::TEMPORARY_REDIRECT)
                .body(Body::new(token.value.clone()))?;

            let cookie_header = HeaderValue::from_str(&cookie_value)?;

            response.headers_mut().insert(SET_COOKIE, cookie_header);

            if let Some(to) = to {
                let sym = if to.contains("?") { "&" } else { "?" };

                response.headers_mut().insert(
                    LOCATION,
                    HeaderValue::from_str(&format!("{}{sym}token={}", to, token.value)).unwrap(),
                );
            } else {
                response
                    .headers_mut()
                    .insert(LOCATION, HeaderValue::from_str("/api/v1/users/me").unwrap());
            }

            Ok(response)
        }

        Err(_) => {
            let mut resp = Response::new(Body::empty());
            *resp.status_mut() = StatusCode::TEMPORARY_REDIRECT;
            Ok(resp)
        }
    }
}

fn sanitize_port(host: &str) -> String {
    match host.split_once(":") {
        Some((domain, _port)) => domain.to_string(),
        None => host.to_string(),
    }
}
