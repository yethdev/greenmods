import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig } from "vite";
import { localeGetter } from "./vite/locales";

const apiProxyTarget = process.env.API_PROXY_TARGET?.trim() || "http://127.0.0.1:4000";

export default defineConfig({
    plugins: [sveltekit(), localeGetter()],
    clearScreen: false,

    server: {
        port: 4001,
        strictPort: true,
        cors: true,
        proxy: {
            "/api": {
                target: apiProxyTarget,
                changeOrigin: true,
            },
        },

        hmr: process.env.REDSTONE_IS_DUMB
            ? {
                clientPort: 443,
                port: 4001,
                protocol: "wss",
                path: "/vite-hmr",
            }
            : {
                path: "/vite-hmr",
            },
    },
});
