import { siteConfig } from "$lib/config";

const normalizePathname = (pathname: string): string => {
    if (!pathname || pathname == "/") {
        return "/";
    }

    return pathname.endsWith("/") ? pathname.slice(0, -1) : pathname;
};

export const absoluteSiteUrl = (path = "/"): string => {
    const pathname = path.startsWith("/") ? path : `/${path}`;

    return new URL(pathname, `${siteConfig.siteUrl}/`).toString();
};

export const canonicalPathForRoute = (
    routeId: string | null | undefined,
    pathname: string,
): string => {
    if (routeId == "/s") {
        return "/s";
    }

    return normalizePathname(pathname);
};

export const robotsDirectiveForRoute = (routeId: string | null | undefined): string => {
    if (!routeId) {
        return "index,follow,max-image-preview:large";
    }

    if (routeId == "/s" || routeId.endsWith("/s")) {
        return "noindex,follow,max-image-preview:large";
    }

    if (
        routeId.startsWith("/admin") ||
        routeId.startsWith("/me") ||
        routeId.startsWith("/new") ||
        routeId.includes("/edit")
    ) {
        return "noindex,nofollow,noarchive";
    }

    return "index,follow,max-image-preview:large";
};

export const trimDescription = (
    value: string | null | undefined,
    fallback: string,
    maxLength = 160,
): string => {
    const cleaned = (value ?? "").replace(/\s+/g, " ").trim() || fallback.trim();

    if (cleaned.length <= maxLength) {
        return cleaned;
    }

    const clipped = cleaned.slice(0, maxLength - 3);
    const lastSpace = clipped.lastIndexOf(" ");
    const safeClip = lastSpace > 80 ? clipped.slice(0, lastSpace) : clipped;

    return `${safeClip.trim()}...`;
};