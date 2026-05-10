<script lang="ts">
    import CollectionCard from "$components/ui/CollectionCard.svelte";
    import { client } from "$lib/api";
    import { siteConfig } from "$lib/config";
    import { absoluteSiteUrl } from "$lib/seo";
    import type { LoadingState } from "$lib/types";
    import { user } from "$lib/user";
    import { onMount } from "svelte";
    import { unwrapOrNull } from "@modhost/api";
    import type { ProjectCollection } from "@modhost/api";
    import Icon from "@iconify/svelte";

    const pageTitle = `Collections | ${siteConfig.siteName}`;
    const pageDescription =
        "Browse curated mod packs, progression lists, and compatibility bundles assembled by the greenmods community.";
    const collectionJsonLd = JSON.stringify({
        "@context": "https://schema.org",
        "@type": "CollectionPage",
        name: "Collections",
        url: absoluteSiteUrl("/collections"),
        description: pageDescription,
        isPartOf: siteConfig.siteUrl,
    });

    let loadingState = $state<LoadingState>("loading");
    let collections = $state<ProjectCollection[]>([]);

    onMount(async () => {
        const data = unwrapOrNull(await client.collections().list()) ?? [];
        collections = data;
        loadingState = data ? "ready" : "failed";
    });
</script>

<svelte:head>
    <title>{pageTitle}</title>
    <meta name="description" content={pageDescription} />
    <meta property="og:title" content={pageTitle} />
    <meta property="og:description" content={pageDescription} />
    <meta name="twitter:title" content={pageTitle} />
    <meta name="twitter:description" content={pageDescription} />
    <script type="application/ld+json">{collectionJsonLd}</script>
</svelte:head>

<div
    class="from-primary-900 via-secondary-700 to-surface-900 m-0 flex w-full flex-col items-center justify-center bg-gradient-to-b from-10% to-90% px-8 py-24 md:px-10 md:py-28"
>
    <div class="mx-auto flex w-full max-w-5xl flex-col items-start gap-6">
        <span class="variant-filled-secondary badge uppercase tracking-[0.2em]">Collections</span>
        <div class="max-w-3xl space-y-3">
            <h1 class="text-4xl font-bold md:text-5xl">Curated mod stacks for every playthrough.</h1>
            <p class="text-lg opacity-80">
                Save a clean progression route, share a streamer setup, or publish a compatibility-tested bundle.
                Collections keep project order, project notes, and direct download paths in one place.
            </p>
        </div>

        <div class="flex flex-wrap gap-3">
            <a href="/s" class="variant-filled-primary btn">
                <Icon icon="tabler:search" width="20" class="mr-2" />
                Browse Mods
            </a>

            {#if $user}
                <a href="/collections/new" class="variant-soft-secondary btn border border-white/10">
                    <Icon icon="tabler:plus" width="20" class="mr-2" />
                    Create Collection
                </a>
            {/if}

            <a href="/" class="variant-soft-secondary btn border border-white/10">
                <Icon icon="tabler:home" width="20" class="mr-2" />
                Back To Home
            </a>
        </div>
    </div>
</div>

<section class="mx-auto w-full max-w-6xl px-4 py-10">
    {#if loadingState == "loading"}
        <p class="text-sm opacity-70">Loading collections...</p>
    {:else if loadingState == "failed"}
        <p class="text-sm opacity-70">Collections could not be loaded right now.</p>
    {:else if collections.length == 0}
        <div class="card p-6">
            <p class="text-lg font-semibold">No collections yet.</p>
            <p class="mt-2 opacity-70">
                Collections appear here as soon as authors start publishing curated mod bundles.
            </p>
        </div>
    {:else}
        <div class="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
            {#each collections as collection}
                <CollectionCard {collection} />
            {/each}
        </div>
    {/if}
</section>