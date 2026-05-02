import { env } from "$env/dynamic/public";

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
    siteName: (env.PUBLIC_APP ?? "greenmods").toLowerCase(),
    tagline: env.PUBLIC_TAGLINE ?? "Subnautica 2 mods that say what they support",
    showBeta: env.PUBLIC_SHOW_BETA == "true",
    type: env.PUBLIC_PKG_TYPE == "packages" ? "packages" : "mods",
    defaultTheme: env.PUBLIC_DEFAULT_THEME ?? "greenmods",
    packageFileFormats: (env.PUBLIC_PKG_FILE_FORMATS ?? ".pak,.ucas,.utoc,.zip,.tar.gz").split(","),
    betaName: env.PUBLIC_GAME_BETA_NAME == "snapshot" ? "snapshot" : "beta",
    themeColor: env.PUBLIC_THEME_COLOR ?? "#16a34a",
    showCommit: env.PUBLIC_SHOW_COMMIT == "true",
    commit: optionalEnv(env.PUBLIC_MODHOST_COMMIT),
    origin: optionalEnv(env.PUBLIC_MODHOST_ORIGIN),
};
