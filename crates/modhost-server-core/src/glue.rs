//! ModHost's setup for [`jsglue`].

use jsglue::{config::GlueConfig, framework::Framework, glue::Glue};
use modhost_config::AppConfig;
use modhost_core::Result;
use std::env;

#[cfg(not(debug_assertions))]
use std::path::PathBuf;

/// Create a new [`Glue`] instance.
/// This will build the UI (see [`modhost_ui::build_ui`]).
#[cfg(debug_assertions)]
pub async fn make_glue(config: &AppConfig) -> Result<Glue> {
    use std::path::PathBuf;

    let dir = format!("{}/../../ui", env!("CARGO_MANIFEST_DIR"));

    modhost_ui::build_ui(config, &PathBuf::from(&dir)).await?;

    Ok(Glue::new(
        GlueConfig::builder()
            .base("http://localhost:4001")
            .project(dir)
            .cmd("bun")
            .arg("run")
            .arg("dev")
            .framework(Framework::Vite("/vite-hmr"))
            .env(config.ui.env())
            .build()?,
    ))
}

/// Create a new [`Glue`] instance.
/// This will build the UI (see [`modhost_ui::build_ui`]).
#[cfg(not(debug_assertions))]
pub async fn make_glue(config: &AppConfig) -> Result<Glue> {
    if let Some(dir) = ui_dir_override() {
        return Ok(Glue::new(
            GlueConfig::builder()
                .dir(dir)
                .base("http://localhost:4001")
                .project(format!("{}/../../ui", env!("CARGO_MANIFEST_DIR")))
                .cmd("bun")
                .arg("run")
                .arg("dev")
                .framework(Framework::Vite("/vite-hmr"))
                .env(config.ui.env())
                .build()?,
        ));
    }

    Ok(Glue::new(
        GlueConfig::builder()
            .dir(modhost_ui::build_ui(config).await?)
            .base("http://localhost:4001")
            .project(format!("{}/../../ui", env!("CARGO_MANIFEST_DIR")))
            .cmd("bun")
            .arg("run")
            .arg("dev")
            .framework(Framework::Vite("/vite-hmr"))
            .env(config.ui.env())
            .build()?,
    ))
}

#[cfg(not(debug_assertions))]
fn ui_dir_override() -> Option<PathBuf> {
    let override_dir = env::var_os("GREENMODS_UI_DIR")
        .map(PathBuf::from)
        .filter(|dir| dir.exists());

    if override_dir.is_some() {
        return override_dir;
    }

    let default_dir = PathBuf::from("/opt/greenmods/ui");
    default_dir.exists().then_some(default_dir)
}
