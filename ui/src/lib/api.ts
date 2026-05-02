import { env } from "$env/dynamic/public";
import { goto } from "$app/navigation";
import { persisted } from "svelte-persisted-store";
import { get } from "svelte/store";
import { ModHostClient } from "@modhost/api";

const trimTrailingSlash = (value: string) => value.replace(/\/+$/, "");

export const apiBase = trimTrailingSlash(env.PUBLIC_API_BASE?.trim() || "/api/v1");

const apiPath = (path: string) => `${apiBase}${path.startsWith("/") ? path : `/${path}`}`;

export const client = new ModHostClient(apiBase);

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
