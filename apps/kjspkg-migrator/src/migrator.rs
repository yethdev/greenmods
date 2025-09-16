use std::{collections::HashMap, env};

use crate::{
    data::get_version_str,
    fetcher::{get_github_owner, get_manifest, get_package_tarball, get_packages_map, get_readme},
};
use anyhow::Result;
use indicatif::ProgressIterator;
use modhost::init_logger;
use modhost_config::get_config;
use modhost_db::{
    ProjectVisibility, create_connection, fresh_migrations, prelude::Users, project_authors,
    project_versions, projects, users, version_files,
};
use object_store::{ObjectStore, PutPayload};
use octocrab::Octocrab;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};
use sha1::{Digest, Sha1};
use tracing::level_filters::LevelFilter;

pub async fn run() -> Result<()> {
    let _ = dotenvy::dotenv();
    let _guard = init_logger("modhost-migrator-kjspkg", LevelFilter::INFO)?;

    let token = env::var("MIGRATOR_TOKEN")
        .expect("Could not find a GitHub token! Is the MIGRATOR_TOKEN environment variable unset?");

    let config = get_config()?;
    let pool = create_connection(Some(config.postgres.uri())).await?;

    fresh_migrations(&pool).await?;

    let pkgs = config.storage.projects()?;
    // let imgs = config.storage.gallery()?;
    let octocrab = Octocrab::builder().personal_token(token).build()?;
    let packages = get_packages_map().await?;
    let mut added_users = HashMap::new();

    for (id, repo) in packages.into_iter().progress() {
        let (owner, mut repo) = repo.split_once('/').unwrap();
        let mut branch: Option<String> = None;
        let mut dir: Option<String> = None;

        if repo.contains('$') {
            let (repo_split, dir_split) = repo.split_once('$').unwrap();

            repo = repo_split;
            dir = Some(dir_split.into());
        }

        if repo.contains('@') {
            let (repo_split, branch_split) = repo.split_once('@').unwrap();

            repo = repo_split;
            branch = Some(branch_split.into());
        }

        if let Some(dir_s) = dir.clone() {
            if dir_s.contains('@') && branch.is_none() {
                let (dir_split, branch_split) = dir_s.split_once('@').unwrap();

                dir = Some(dir_split.into());
                branch = Some(branch_split.into());
            }
        }

        // let branch_or_default = branch.clone().unwrap_or("[default branch]".into());

        // info!("Uploading {owner}/{repo} (path {dir:?}) at branch {branch_or_default:?}");

        let (author_name, author_id) = get_github_owner(&octocrab, owner, repo).await?;
        let manifest = get_manifest(&octocrab, owner, repo, &branch, &dir).await?;

        if let Some(manifest) = manifest {
            let readme = get_readme(&octocrab, owner, repo, &branch)
                .await
                .unwrap_or("During migration from the old KJSPKG, no README was found.".into());

            let (commit, tarball) =
                get_package_tarball(&octocrab, owner, repo, &branch, &dir).await?;

            if let std::collections::hash_map::Entry::Vacant(e) = added_users.entry(author_id) {
                let user = users::ActiveModel {
                    github_id: Set(author_id as i32),
                    username: Set(author_name),
                    admin: Set(false),
                    moderator: Set(false),
                    ..Default::default()
                };

                Users::insert(user)
                    .on_conflict_do_nothing()
                    .exec(&pool)
                    .await?;

                let user = Users::find()
                    .filter(users::Column::GithubId.eq(author_id as i32))
                    .one(&pool)
                    .await?
                    .unwrap();

                e.insert(user.id);
            }

            let user_id = *added_users.get(&author_id).unwrap();

            let project = projects::ActiveModel {
                slug: Set(id.clone()),
                name: Set(id.clone()),
                description: Set(manifest.description),
                issues: Set(Some(format!(
                    "https://github.com/{}/{}/issues",
                    owner, repo
                ))),
                source: Set(Some(format!("https://github.com/{}/{}", owner, repo))),
                wiki: Set(Some(format!("https://github.com/{}/{}/wiki", owner, repo))),
                license: Set(None),
                readme: Set(readme),
                tags: Set(Vec::new()),
                visibility: Set(ProjectVisibility::Public),
                ..Default::default()
            }
            .insert(&pool)
            .await?;

            project_authors::ActiveModel {
                project: Set(project.id),
                user_id: Set(user_id),
            }
            .insert(&pool)
            .await?;

            let mut hasher = Sha1::new();

            hasher.update(&tarball);

            let file_id = format!("{:x}", hasher.finalize());
            let file_size = tarball.len() as i64;

            pkgs.put(
                &format!("/{}", &file_id).into(),
                PutPayload::from_bytes(tarball.into()),
            )
            .await?;

            let version = project_versions::ActiveModel {
                name: Set((&commit[0..7]).into()),
                version_number: Set(format!("0.0.0+{}", &commit[0..7])),
                changelog: Set(Some("Migrated from the old KJSPKG.".into())),
                downloads: Set(0),
                loaders: Set(manifest.modloaders),
                project: Set(project.id),
                game_versions: Set(manifest
                    .versions
                    .into_iter()
                    .filter_map(get_version_str)
                    .collect()),
                ..Default::default()
            }
            .insert(&pool)
            .await?;

            version_files::ActiveModel {
                version_id: Set(version.id),
                file_name: Set(format!("{}-{}.tar.gz", id, &commit[0..7])),
                s3_id: Set(file_id.clone()),
                sha1: Set(file_id),
                size: Set(file_size),
                ..Default::default()
            }
            .insert(&pool)
            .await?;
        }
    }

    Ok(())
}
