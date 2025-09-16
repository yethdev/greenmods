use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use utoipa::{ToResponse, ToSchema};

pub use modhost_entities::*;

/// An item in the moderation queue.
pub type ModerationQueueItem = moderation_queue::Model;

/// The status of an item in the moderation queue.
pub type ModerationQueueStatus = sea_orm_active_enums::ModerationStatusEnum;

/// A moderation comment.
pub type ModerationComment = moderation_comment::Model;

/// A project's visibility.
pub type ProjectVisibility = sea_orm_active_enums::VisibilityEnum;

/// A gallery image.
pub type GalleryImage = gallery_images::Model;

/// A user.
pub type User = users::Model;

/// A user's token.
pub type UserToken = user_tokens::Model;

/// A project.
pub type Project = projects::Model;

/// A project version.
pub type ProjectVersion = project_versions::Model;

/// A version file.
pub type ProjectFile = version_files::Model;

/// A project author entry.
pub type ProjectAuthor = project_authors::Model;

/// A project relation entry.
pub type ProjectRelation = project_relations::Model;

/// A reference to a project version.
pub type ProjectVersionRef = project_version_refs::Model;

/// A gallery image, modified for public consumption (i.e. REST endpoints).
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, ToSchema, ToResponse,
)]
pub struct PublicGalleryImage {
    /// The gallery image ID.
    pub id: i32,

    /// The project ID.
    pub project: i32,

    /// The display name of the version.
    pub name: String,

    /// A URL to access this image with.
    pub url: String,

    /// An optional markdown-formatted description.
    pub description: Option<String>,

    /// The order of this image.
    pub ordering: i32,

    /// The date this version was created.
    pub created_at: NaiveDateTime,

    /// The date this version was last updated.
    pub updated_at: NaiveDateTime,
}

/// A project with additional data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, ToResponse)]
pub struct ProjectData {
    /// The project's ID.
    pub id: i32,

    /// The project's name.
    pub name: String,

    /// The project's URL slug.
    pub slug: String,

    /// The project's README.
    pub readme: String,

    /// A short description of the project.
    pub description: String,

    /// An optional link to the project's source code.
    pub source: Option<String>,

    /// An optional link to the project's issue tracker.
    pub issues: Option<String>,

    /// An optional link to the project's wiki.
    pub wiki: Option<String>,

    /// The date the project was created.
    pub created_at: NaiveDateTime,

    /// The date the project was last updated.
    pub updated_at: NaiveDateTime,

    /// The number of downloads the project has.
    pub downloads: i32,

    /// This project's authors.
    pub authors: Vec<User>,

    /// The visibility of a project.
    pub visibility: ProjectVisibility,

    /// The license the project is under.
    pub license: Option<String>,

    /// A list of tags for this project.
    pub tags: Vec<String>,
}

/// A manifest for a project.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, ToSchema, ToResponse, Serialize, Deserialize,
)]
pub struct ProjectManifest {
    /// The project name
    pub name: String,

    /// The project authors
    pub authors: Vec<String>,

    /// The project version
    pub version: String,

    /// The project description
    pub description: String,

    /// The loaders this project works on
    pub loaders: Vec<String>,

    /// The game versions this project works on
    pub game_versions: Vec<String>,

    /// This project's dependencies
    pub dependencies: Vec<String>,

    /// This project's incompatibilities
    pub incompatibilities: Vec<String>,
}

/// A helper for converting a [`Project`] into a [`ProjectData`].
pub trait AsProjectData {
    /// Turn this into a [`ProjectData`] by providing a list of [`User`]s.
    fn with_authors(self, authors: Vec<User>) -> ProjectData;
}

impl AsProjectData for Project {
    fn with_authors(self, authors: Vec<User>) -> ProjectData {
        ProjectData {
            id: self.id,
            name: self.name,
            slug: self.slug,
            readme: self.readme,
            description: self.description,
            source: self.source,
            issues: self.issues,
            wiki: self.wiki,
            created_at: self.created_at,
            updated_at: self.updated_at,
            downloads: self.downloads,
            visibility: self.visibility,
            license: self.license,
            tags: self.tags,
            authors,
        }
    }
}

impl ProjectData {
    /// Turn this into a [`Project`].
    pub fn into_project(self) -> Project {
        Project {
            id: self.id,
            name: self.name,
            slug: self.slug,
            readme: self.readme,
            description: self.description,
            source: self.source,
            issues: self.issues,
            wiki: self.wiki,
            created_at: self.created_at,
            updated_at: self.updated_at,
            downloads: self.downloads,
            visibility: self.visibility,
            license: self.license,
            tags: self.tags,
        }
    }
}

/// A project version with a list of its files.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema, ToResponse)]
pub struct ProjectVersionData {
    /// The project version ID.
    pub id: i32,

    /// The project ID.
    pub project: i32,

    /// The display name of the version.
    pub name: String,

    /// The version number.
    pub version_number: String,

    /// An optional markdown-formatted changelog.
    pub changelog: Option<String>,

    /// A list of loaders this version works on.
    pub loaders: Vec<String>,

    /// A list of game versions this works on.
    pub game_versions: Vec<String>,

    /// The date this version was created.
    pub created_at: NaiveDateTime,

    /// The date this version was last updated.
    pub updated_at: NaiveDateTime,

    /// The number of downloads this version has.
    pub downloads: i32,

    /// This version's files.
    pub files: Vec<ProjectFile>,
}

/// A trait to convert a [`ProjectVersion`] into [`ProjectVersionData`].
pub trait AsVersionData {
    /// Transform this into [`ProjectVersionData`] with a list of [`ProjectFile`]s.
    fn with_files(self, files: Vec<ProjectFile>) -> ProjectVersionData;
}

impl AsVersionData for ProjectVersion {
    fn with_files(self, files: Vec<ProjectFile>) -> ProjectVersionData {
        ProjectVersionData {
            id: self.id,
            project: self.project,
            name: self.name,
            version_number: self.version_number,
            changelog: self.changelog,
            loaders: self.loaders,
            game_versions: self.game_versions,
            created_at: self.created_at,
            updated_at: self.updated_at,
            downloads: self.downloads,
            files,
        }
    }
}
