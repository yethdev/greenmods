<script lang="ts">
    import { _ } from "svelte-i18n";
    import { AppBar, getDrawerStore } from "@skeletonlabs/skeleton";
    import { currentQuery, currentScrollPosition } from "$lib/state";
    import { page } from "$app/state";
    import { fly } from "svelte/transition";
    import IconAuth from "$components/auth/IconAuth.svelte";
    import { onMount } from "svelte";
    import { goto, replaceState } from "$app/navigation";
    import { siteConfig } from "$lib/config";
    import Icon from "@iconify/svelte";

    let inputElement: HTMLInputElement = $state(null!);
    const drawerStore = getDrawerStore();
    const searchRouteIds = new Set(["/s", "/one/s", "/zero/s"]);
    const isSearchRoute = (routeId: string | null | undefined) => searchRouteIds.has(routeId ?? "");

    onMount(() => {
        $currentQuery = isSearchRoute(page.route.id) ? (page.url.searchParams.get("q") ?? "") : "";
    });

    const updateQuery = async () => {
        const targetPath = isSearchRoute(page.route.id) ? page.url.pathname : "/s";

        if (!isSearchRoute(page.route.id)) {
            await goto(targetPath, { keepFocus: true });
        }

        if ($currentQuery != "") page.url.searchParams.set("q", $currentQuery);
        else page.url.searchParams.delete("q");

        replaceState(page.url, page.state);
    };

    const openHomeDrawer = () => {
        drawerStore.open({
            id: "home",
            bgDrawer: "bg-surface-800 text-white",
            width: "w-[300px]",
            rounded: "rounded-none",
        });
    };
</script>

<AppBar
    gridColumns="grid-cols-[auto_1fr_auto]"
    slotDefault="place-self-center !w-full"
    slotTrail="place-self-end"
    class="vt-none justify-center transition-colors"
    background={$currentScrollPosition.y > 16 ? "bg-surface-800/75" : "bg-transparent"}
>
    {#snippet lead()}
        <button
            type="button"
            onclick={openHomeDrawer}
            class="mr-2 flex items-center"
            aria-label="Open navigation"
        >
            <Icon icon="tabler:menu-2" height="28" />
        </button>

        <a class="flex items-center gap-2" href="/">
            <img src="/modhost.png" alt="logo" class="rounded-token aspect-square w-8 min-w-8" />
            <span class="hidden lg:inline">{siteConfig.siteName}</span>

            {#if siteConfig.showBeta}
                <span class="variant-filled-secondary badge">{$_("site.beta")}</span>
            {/if}
        </a>
    {/snippet}

    {#snippet trail()}
        <IconAuth />
    {/snippet}

    <div class="flex flex-row items-center justify-start">
        <div
            class="input-group input-group-divider w-full grid-cols-[1fr] transition-all lg:grid-cols-[auto_1fr]"
            transition:fly={{ y: -40 }}
        >
            <a href="/s" class="text-surface-400 hidden lg:inline">
                <Icon icon="tabler:search" height="24" class="hidden lg:block" />
            </a>

            <input
                type="search"
                class="w-full transition-all"
                placeholder={$_(`search.placeholder.${siteConfig.type}`)}
                aria-label={$_(`search.placeholder.${siteConfig.type}`)}
                bind:this={inputElement}
                bind:value={$currentQuery}
                oninput={updateQuery}
                onchange={updateQuery}
            />
        </div>
    </div>
</AppBar>
