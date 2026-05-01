use modhost::GameVersion;

pub fn known_versions() -> Vec<GameVersion> {
    [
        ("early-access-2026.05", false),
        ("early-access", false),
        ("preview", true),
    ]
    .into_iter()
    .map(|(id, beta)| GameVersion {
        id: id.into(),
        beta,
    })
    .collect()
}
