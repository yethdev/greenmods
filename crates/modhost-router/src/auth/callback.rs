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
use axum_extra::extract::{CookieJar, Host};
use modhost_core::Result;
use modhost_db::{create_token, prelude::Users, users};
use modhost_middleware::scheme::Scheme;
use modhost_server_core::{github::create_github_client, state::AppState};
use oauth2::{RedirectUrl, TokenResponse};
use sea_orm::{ActiveValue::Set, EntityTrait, sea_query::OnConflict};
use std::collections::HashMap;

use super::{
    CALLBACK_URL,
    login::{OAUTH_STATE_COOKIE, expire_state_cookie, safe_redirect_path},
};

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
    jar: CookieJar,
    State(state): State<AppState>,
    Host(host): Host,
    Scheme(scheme): Scheme,
    url: Uri,
) -> Result<Response> {
    let query = url::form_urlencoded::parse(url.query().unwrap_or_default().as_bytes())
        .into_owned()
        .collect::<HashMap<String, String>>();

    let Some(code) = query.get("code") else {
        return bad_request("Missing GitHub OAuth code.");
    };

    let Some(oauth_state) = query.get("state") else {
        return bad_request("Missing GitHub OAuth state.");
    };

    if jar
        .get(OAUTH_STATE_COOKIE)
        .map(|cookie| cookie.value() != oauth_state)
        .unwrap_or(true)
    {
        return bad_request("Invalid GitHub OAuth state.");
    }

    let to = query.get("to").map(|to| safe_redirect_path(to));

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

            let cookie_value = auth_cookie(&token.value, &scheme);

            let mut response = Response::builder()
                .status(StatusCode::TEMPORARY_REDIRECT)
                .body(Body::empty())?;

            let cookie_header = HeaderValue::from_str(&cookie_value)?;

            response.headers_mut().append(SET_COOKIE, cookie_header);
            response.headers_mut().append(
                SET_COOKIE,
                HeaderValue::from_str(&expire_state_cookie(&scheme))?,
            );

            if let Some(to) = to {
                response
                    .headers_mut()
                    .insert(LOCATION, HeaderValue::from_str(&to).unwrap());
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

fn bad_request(message: &'static str) -> Result<Response> {
    Ok(Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(Body::new(message.to_string()))?)
}

fn auth_cookie(value: &str, scheme: &str) -> String {
    let secure = if scheme == "https" { "; Secure" } else { "" };

    format!("auth-token={value}; Max-Age=604800; HttpOnly; SameSite=Lax; Path=/{secure}")
}
