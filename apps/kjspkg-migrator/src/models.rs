use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyManifest {
    pub author: String,
    pub description: String,
    pub versions: Vec<u16>,
    pub modloaders: Vec<String>,
    pub dependencies: Option<Vec<String>>,
    pub incompatibilities: Option<Vec<String>>,
}
