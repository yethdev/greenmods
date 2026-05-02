/// <reference path="../app.d.ts" />

import {
    PUBLIC_APP,
    PUBLIC_DEFAULT_THEME,
    PUBLIC_GAME_BETA_NAME,
    PUBLIC_PKG_FILE_FORMATS,
    PUBLIC_PKG_TYPE,
    PUBLIC_SHOW_BETA,
    PUBLIC_TAGLINE,
    PUBLIC_THEME_COLOR,
    PUBLIC_SHOW_COMMIT,
    PUBLIC_MODHOST_COMMIT,
    PUBLIC_MODHOST_ORIGIN,
} from "$env/static/public";

export interface SiteConfig {
    siteName: string;
    tagline: string;
    showBeta: boolean;
    type: "mods" | "packages";
    defaultTheme: string;
    packageFileFormats: string[];
    betaName: "beta" | "snapshot";
    themeColor: string;
    showCommit: boolean;
    commit?: string;
    origin?: string;
}

const optionalEnv = (value: string | undefined): string | undefined => {
    const trimmed = value?.trim() ?? "";
    return trimmed == "" ? undefined : trimmed;
};

export const siteConfig: SiteConfig = {
    siteName: PUBLIC_APP ?? "GreenMods",
    tagline: PUBLIC_TAGLINE ?? "Subnautica 2 mods that say what they support",
    showBeta: PUBLIC_SHOW_BETA == "true",
    type: PUBLIC_PKG_TYPE == "packages" ? "packages" : "mods",
    defaultTheme: PUBLIC_DEFAULT_THEME ?? "greenmods",
    packageFileFormats: (PUBLIC_PKG_FILE_FORMATS ?? ".pak,.ucas,.utoc,.zip,.tar.gz").split(","),
    betaName: PUBLIC_GAME_BETA_NAME == "snapshot" ? "snapshot" : "beta",
    themeColor: PUBLIC_THEME_COLOR ?? "#16a34a",
    showCommit: PUBLIC_SHOW_COMMIT == "true",
    commit: optionalEnv(PUBLIC_MODHOST_COMMIT),
    origin: optionalEnv(PUBLIC_MODHOST_ORIGIN),
};
