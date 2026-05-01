//! The route to initiate the GitHub login flow.

use std::collections::HashMap;

use super::CALLBACK_URL;
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
use modhost_middleware::scheme::Scheme;
use modhost_server_core::state::AppState;
use oauth2::{CsrfToken, RedirectUrl, Scope};
use url::Url;

pub(super) const OAUTH_STATE_COOKIE: &str = "gm-oauth-state";

/// GitHub Auth Login
///
/// Initiate the GitHub login flow.
#[utoipa::path(
    get,
    path = "/github/login",
    tag = "Auth",
    params(
        ("redirect_uri" = Option<String>, Query, description = "An optional custom URL to redirect to."),
    ),
    responses(
        (status = 307, description = "Redirecting to GitHub for login"),
    ),
)]
#[debug_handler]
pub async fn login_handler(
    State(state): State<AppState>,
    Host(host): Host,
    Scheme(scheme): Scheme,
    url: Uri,
) -> Result<Response> {
    let query = url::form_urlencoded::parse(url.query().unwrap_or_default().as_bytes())
        .into_owned()
        .collect::<HashMap<String, String>>();

    let callback_url_base =
        safe_redirect_path(query.get("redirect_uri").map(String::as_str).unwrap_or("/"));

    let callback_url = callback_url(&scheme, &host, &callback_url_base)?;

    let client = state
        .auth
        .set_redirect_uri(RedirectUrl::new(callback_url).unwrap());

    let (authorize_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("user:email".to_string()))
        .add_scope(Scope::new("read:user".to_string()))
        .url();

    // authorize_url
    //     .query_pairs_mut()
    //     .append_pair("prompt", "consent");

    let mut resp = Response::new(Body::empty());

    resp.headers_mut()
        .insert(LOCATION, HeaderValue::from_str(authorize_url.as_str())?);

    resp.headers_mut().insert(
        SET_COOKIE,
        HeaderValue::from_str(&state_cookie(csrf_token.secret(), &scheme, 600))?,
    );

    *resp.status_mut() = StatusCode::TEMPORARY_REDIRECT;

    Ok(resp)
}

pub(super) fn safe_redirect_path(value: &str) -> String {
    if value.starts_with('/')
        && !value.starts_with("//")
        && !value.contains('\\')
        && value.bytes().all(|byte| (0x21..=0x7e).contains(&byte))
    {
        value.into()
    } else {
        "/".into()
    }
}

fn state_cookie(value: &str, scheme: &str, max_age: u64) -> String {
    let secure = if scheme == "https" { "; Secure" } else { "" };

    format!(
        "{OAUTH_STATE_COOKIE}={value}; Max-Age={max_age}; HttpOnly; SameSite=Lax; Path={CALLBACK_URL}{secure}"
    )
}

pub(super) fn expire_state_cookie(scheme: &str) -> String {
    let secure = if scheme == "https" { "; Secure" } else { "" };

    format!("{OAUTH_STATE_COOKIE}=; Max-Age=0; HttpOnly; SameSite=Lax; Path={CALLBACK_URL}{secure}")
}

fn callback_url(scheme: &str, host: &str, to: &str) -> Result<String> {
    let mut url = Url::parse(&format!("{scheme}://{host}{CALLBACK_URL}"))?;

    url.query_pairs_mut().append_pair("to", to);

    Ok(url.to_string())
}
