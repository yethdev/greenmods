use modhost::{Tag, tags};

pub fn tags() -> Vec<Tag> {
    tags![
        "tested", "Tested", "tabler:shield-check";
        "needs-testing", "Needs testing", "tabler:flask";
        "co-op-ready", "Co-op ready", "tabler:users";
        "single-player", "Single-player", "tabler:user";
        "client-side", "Client-side", "tabler:device-desktop";
        "server-side", "Server-side", "tabler:server";
        "gameplay", "Gameplay", "tabler:device-gamepad-2";
        "quality-of-life", "Quality of life", "tabler:adjustments";
        "performance", "Performance", "tabler:gauge";
        "visuals", "Visuals", "tabler:palette";
        "audio", "Audio", "tabler:volume";
        "base-building", "Base building", "tabler:building";
        "vehicles", "Vehicles", "tabler:submarine";
        "creatures", "Creatures", "tabler:fish";
        "biomes", "Biomes", "tabler:map";
        "ui", "UI", "tabler:layout";
        "translations", "Translations", "tabler:language";
        "tools", "Tools", "tabler:tool";
        "save-safe", "Save safe", "tabler:database-check";
        "requires-ue4ss", "Requires UE4SS", "tabler:plug-connected";
        "pak-only", "Pak only", "tabler:package";
    ]
}
