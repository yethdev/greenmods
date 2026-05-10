import { env } from "$env/dynamic/public";
import { goto } from "$app/navigation";
import { persisted } from "svelte-persisted-store";
import { get } from "svelte/store";
import { ModHostClient, unwrapOrNull } from "@modhost/api";

const trimTrailingSlash = (value: string) => value.replace(/\/+$/, "");
const GALLERY_PREVIEW_TTL_MS = 5 * 60 * 1000;

export const apiBase = trimTrailingSlash(env.PUBLIC_API_BASE?.trim() || "/api/v1");

const apiPath = (path: string) => `${apiBase}${path.startsWith("/") ? path : `/${path}`}`;

export const client = new ModHostClient(apiBase);

const galleryPreviewCache = new Map<
    string,
    { expiresAt: number; promise: Promise<string | undefined> }
>();

const tokenStore = persisted<string | undefined>("auth-token", undefined);

export const setToken = (token?: string) => {
    tokenStore.set(token);
    checkClientToken();
};

export const getToken = () => get(tokenStore);
export const isLoggedIn = () => !!getToken();

export const checkClientToken = () => {
    if (isLoggedIn() && !client.hasToken()) client.setToken(getToken()!);
    else if (!isLoggedIn() && client.hasToken()) client.unsetToken();
};

export const beginLogin = (redirect: string) =>
    goto(
        redirect
            ? apiPath(`/auth/github/login?redirect_uri=${encodeURIComponent(redirect)}`)
            : apiPath("/auth/github/login"),
    );

export const getProjectPreviewImage = async (
    project: string | number,
): Promise<string | undefined> => {
    const key = project.toString();
    const now = Date.now();
    const cached = galleryPreviewCache.get(key);

    if (cached && cached.expiresAt > now) {
        return cached.promise;
    }

    const promise = (async () => {
        const gallery = unwrapOrNull(await client.project(project).gallery().list()) ?? [];
        return gallery[0]?.url;
    })();

    galleryPreviewCache.set(key, {
        expiresAt: now + GALLERY_PREVIEW_TTL_MS,
        promise,
    });

    return promise;
};

export const invalidateProjectPreviewImage = (project?: string | number) => {
    if (project == undefined) {
        galleryPreviewCache.clear();
        return;
    }

    galleryPreviewCache.delete(project.toString());
};
