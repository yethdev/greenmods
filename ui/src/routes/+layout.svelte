<script lang="ts">
    import "../app.pcss";
    import "highlight.js/styles/github-dark.min.css";

    import "carta-md/default.css";
    import { currentScrollPosition } from "$lib/state";
    import {
        Modal,
        Toast,
        storePopup,
        initializeStores,
        ProgressRadial,
    } from "@skeletonlabs/skeleton";
    import type { ModalComponent } from "@skeletonlabs/skeleton";
    import { computePosition, autoUpdate, offset, shift, flip, arrow } from "@floating-ui/dom";
    import { afterNavigate, beforeNavigate, onNavigate } from "$app/navigation";
    import { onMount } from "svelte";
    import type { Snippet } from "svelte";
    import { fly } from "svelte/transition";
    import HeaderBar from "$components/ui/HeaderBar.svelte";
    import ContextMenu from "$components/ui/ContextMenu.svelte";
    import { page } from "$app/stores";
    import Drawers from "$components/ui/Drawers.svelte";
    import { siteConfig } from "$lib/config";
    import ConfirmDeleteModal from "$components/modals/ConfirmDeleteModal.svelte";
    import ConfirmDeleteVersionModal from "$components/modals/ConfirmDeleteVersionModal.svelte";
    import ConfirmDeleteImageModal from "$components/modals/ConfirmDeleteImageModal.svelte";
    import type { NavigationTarget } from "@sveltejs/kit";
    import { editRoutes, pkgRoutes } from "$lib/routes";
    import ImageViewModal from "$components/modals/ImageViewModal.svelte";
    import { setToken } from "$lib/api";
    import { updateUser, userPreferencesStore } from "$lib/user";
    import { initMeta } from "$lib/meta";
    import Popups from "$components/ui/Popups.svelte";
    import ConfirmDeleteGenericModal from "$components/modals/ConfirmDeleteGenericModal.svelte";
    import {
        absoluteSiteUrl,
        canonicalPathForRoute,
        robotsDirectiveForRoute,
    } from "$lib/seo";

    const { children }: { children: Snippet } = $props();
    const siteDescription =
        "Open mod hosting for Subnautica, Below Zero, and Subnautica 2 with GitHub sync, collections, and compatibility notes.";
    let navigating = $state(false);

    const modalRegistry: Record<string, ModalComponent> = {
        confirmDelete: { ref: ConfirmDeleteModal },
        confirmDeleteImage: { ref: ConfirmDeleteImageModal },
        confirmDeleteVersion: { ref: ConfirmDeleteVersionModal },
        confirmDeleteGeneric: { ref: ConfirmDeleteGenericModal },
        imageView: { ref: ImageViewModal },
    };

    storePopup.set({ computePosition, autoUpdate, offset, shift, flip, arrow });
    initializeStores();

    const handleScroll = (e: Event) =>
        ($currentScrollPosition = {
            x: (e.currentTarget as Element).scrollLeft,
            y: (e.currentTarget as Element).scrollTop,
        });

    onMount(async () => {
        if ($page.url.searchParams.has("token")) {
            const token = $page.url.searchParams.get("token");
            setToken(token!);
            $page.url.searchParams.delete("token");
            history.replaceState(null, "", $page.url.toString());
        }

        if ($userPreferencesStore.lightMode) document.documentElement.classList.remove("dark");

        document.body.dataset.theme = $userPreferencesStore.theme ?? siteConfig.defaultTheme;

        (async () => {
            await Promise.all([initMeta(), updateUser()]);
        })();
    });

    beforeNavigate(() => (navigating = true));
    afterNavigate(() => (navigating = false));

    const isPackageRoute = (to?: NavigationTarget, from?: NavigationTarget) =>
        pkgRoutes.includes(to?.route.id ?? "") && pkgRoutes.includes(from?.route.id ?? "");

    const isEditRoute = (to?: NavigationTarget, from?: NavigationTarget) =>
        editRoutes.includes(to?.route.id ?? "") && editRoutes.includes(from?.route.id ?? "");

    onNavigate((navigation) => {
        if (
            navigation.to?.route.id == navigation.from?.route.id ||
            isPackageRoute(navigation.to ?? undefined, navigation.from ?? undefined) ||
            isEditRoute(navigation.to ?? undefined, navigation.from ?? undefined)
        )
            return;

        if (!document.startViewTransition) return;

        return new Promise((resolve) => {
            document.startViewTransition(async () => {
                resolve();
                await navigation.complete;
            });
        });
    });

    const canonicalHref = $derived(
        absoluteSiteUrl(canonicalPathForRoute($page.route.id, $page.url.pathname)),
    );
    const robotsDirective = $derived(robotsDirectiveForRoute($page.route.id));
    const openGraphType = $derived($page.route.id?.startsWith("/p/") ? "article" : "website");
    const socialImageUrl = absoluteSiteUrl("/modhost.png");
</script>

<svelte:head>
    <title>{siteConfig.siteName}</title>
    <link rel="canonical" href={canonicalHref} />
    <meta name="description" content={siteDescription} />
    <meta name="application-name" content={siteConfig.siteName} />
    <meta name="robots" content={robotsDirective} />
    <meta name="googlebot" content={robotsDirective} />
    <meta property="og:title" content={siteConfig.siteName} />
    <meta property="og:site_name" content={siteConfig.siteName} />
    <meta property="og:type" content={openGraphType} />
    <meta property="og:url" content={canonicalHref} />
    <meta property="og:image" content={socialImageUrl} />
    <meta property="og:image:alt" content="Preview of the greenmods mod library" />
    <meta property="og:description" content={siteDescription} />
    <meta property="og:locale" content="en_US" />
    <meta name="twitter:card" content="summary_large_image" />
    <meta name="twitter:title" content={siteConfig.siteName} />
    <meta name="twitter:description" content={siteDescription} />
    <meta name="twitter:image" content={socialImageUrl} />
    <meta name="twitter:image:alt" content="Preview of the greenmods mod library" />
    <meta name="theme-color" content={siteConfig.themeColor} />
</svelte:head>

<Toast position="br" max={8} />
<ContextMenu />
<Drawers />
<Modal components={modalRegistry} />
<Popups />

{#if navigating}
    <div
        class="card variant-soft-success absolute bottom-8 right-8 z-40 flex w-16 p-4"
        in:fly={{ x: 40, delay: 200 }}
        out:fly={{ x: 40 }}
    >
        <ProgressRadial stroke={40} meter="stroke-success-500" track="stroke-success-500/30" />
    </div>
{/if}

<div class="flex h-full w-full flex-col overflow-hidden">
    <header class="flex-none">
        <HeaderBar />
    </header>

    <div
        class="flex h-full w-full flex-col overflow-x-hidden"
        style:scrollbar-gutter="auto"
        onscroll={handleScroll}
    >
        <main class="flex-auto">
            {#if $page.route.id == "/" || $page.route.id == "/one" || $page.route.id == "/zero"}
                <div class="container flex min-h-full w-full max-w-full flex-col">
                    {#key $page.url.href}
                        {@render children?.()}
                    {/key}
                </div>
            {:else}
                <div
                    class="container mx-auto flex min-h-full max-w-screen-lg flex-col space-y-2 p-4"
                >
                    {#key $page.url.href}
                        {@render children?.()}
                    {/key}
                </div>
            {/if}
        </main>

        <footer class="flex w-full flex-row items-center justify-between p-2">
            <span class="hidden md:inline">
                <a
                    href="https://github.com/yethdev/greenmods"
                    class="anchor no-underline"
                    target="_blank"
                    rel="noopener noreferrer">GitHub</a
                >
                &bull;
                <a
                    href="https://github.com/yethdev/greenmods/wiki"
                    class="anchor no-underline"
                    target="_blank"
                    rel="noopener noreferrer">Wiki</a
                >
                &bull;
                <a
                    href="/api/v1/docs/scalar"
                    class="anchor no-underline"
                    target="_blank"
                    rel="noopener noreferrer"
                    >API Docs</a
                >
            </span>

            <span class="mt-auto hidden text-sm opacity-50 md:inline">
                Powered by
                <a
                    href="https://github.com/RedstoneWizard08/ModHost"
                    class="anchor no-underline"
                    target="_blank"
                    rel="noopener noreferrer">ModHost</a
                >
                {#if siteConfig.showCommit && siteConfig.origin && siteConfig.commit}
                    (<a
                        href="{siteConfig.origin}/commit/{siteConfig.commit}"
                        class="anchor no-underline"
                        target="_blank"
                        rel="noopener noreferrer">{siteConfig.commit}</a
                    >)
                {/if}
            </span>
        </footer>
    </div>
</div>
