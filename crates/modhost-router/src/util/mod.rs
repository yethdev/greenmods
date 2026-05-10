//! Utilities.

pub mod health;
pub mod stats;

/// The spec for utility endpoints.
#[derive(OpenApi)]
#[openapi(paths(health::livez_handler, health::readyz_handler,))]
pub struct UtilityApi;
