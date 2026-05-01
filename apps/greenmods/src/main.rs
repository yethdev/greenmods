quickhost::quickhost! {
    versions = [greenmods::vers::known_versions()];
    loaders = [modhost::loaders!["UE4SS", "Paks", "BepInEx", "Manual"]];
    tags = [greenmods::tags::tags()];
    verifier = [greenmods::verify::verify_upload];
}
