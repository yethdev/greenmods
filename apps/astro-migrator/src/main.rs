use anyhow::Result;
use astro_migrator::models::{Mod, ModsDump};
use indicatif::ProgressIterator;
use itertools::Itertools;
use modhost::init_logger;
use modhost_config::get_config;
use modhost_db::{create_connection, fresh_migrations, users};
use modhost_search::MeilisearchService;
use ron::ser::PrettyConfig;
use sea_orm::ActiveModelTrait;
use sea_orm::ActiveValue::Set;
use std::{fs, path::PathBuf};
use tracing::level_filters::LevelFilter;

#[tokio::main]
pub async fn main() -> Result<()> {
    let _guard = init_logger("modhost-migrator-astro", LevelFilter::INFO)?;

    let config = get_config()?;
    let db = create_connection(Some(config.postgres.uri())).await?;

    fresh_migrations(&db).await?;

    let pkgs = config.storage.projects()?;
    let imgs = config.storage.gallery()?;

    let mods_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("mods");
    let raw = fs::read_to_string(mods_dir.join("mods.json"))?;
    let tags_file = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tags.ron");
    let data = serde_json::from_str::<ModsDump>(&raw)?;
    let dump: Vec<Mod> = data.into();

    let user = users::ActiveModel {
        github_id: Set(-1),
        username: Set("ModHost Migrator".into()),
        admin: Set(false),
        moderator: Set(false),
        ..Default::default()
    }
    .insert(&db)
    .await?;

    let id = user.id;

    let mut tags = Vec::new();

    for item in dump.into_iter().progress() {
        let (pkg, _) = item.upload_all(id, &db, &pkgs, &imgs).await?;

        tags.extend(pkg.tags);
    }

    let tags = tags.into_iter().sorted().dedup().collect_vec();

    let search = MeilisearchService::new(&config)?;

    search.index_projects(&db).await?;

    fs::write(
        tags_file,
        ron::ser::to_string_pretty(&tags, PrettyConfig::default())?,
    )?;

    println!("Known tags written to tags.ron");

    Ok(())
}
