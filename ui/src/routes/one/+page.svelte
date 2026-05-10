<script lang="ts">
    import ProjectScroller from "$components/ui/ProjectScroller.svelte";
    import { siteConfig } from "$lib/config";
    import { absoluteSiteUrl } from "$lib/seo";
    import type { Facet } from "@modhost/api";
    import Icon from "@iconify/svelte";

    const subnauticaOneFilters: Facet[] = [["tags", ["subnautica-1"]]];
    const browseHref = "/one/s";
    const pageTitle = `Subnautica 1 Library | ${siteConfig.siteName}`;
    const pageDescription =
        "Browse Subnautica 1 mods imported into the greenmods library with source attribution and compatibility data.";
    const collectionJsonLd = JSON.stringify({
        "@context": "https://schema.org",
        "@type": "CollectionPage",
        name: "Subnautica 1 Library",
        url: absoluteSiteUrl("/one"),
        description: pageDescription,
        isPartOf: siteConfig.siteUrl,
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
    class="from-primary-900 via-secondary-700 to-surface-900 m-0 flex w-full flex-col items-center justify-center bg-gradient-to-b from-10% to-90% px-8 py-32 md:px-10 md:py-36"
>
    <h1 class="flex flex-row items-center justify-center text-3xl md:text-4xl">
        <span>Subnautica 1 Library</span>
    </h1>

    <span class="mt-3 text-center text-xl font-bold">
        Mods for Legacy and Living Large (2.0).
    </span>

    <div class="mt-12 flex flex-col items-center justify-center gap-4 md:flex-row">
        <a href={browseHref} class="variant-filled-primary btn">
            <span><Icon icon="tabler:search" height="24" class="mr-2" /></span>
            <span>Browse Mods</span>
        </a>

        <a href="/" class="variant-soft-secondary btn border border-white/10">
            <span><Icon icon="tabler:arrow-left" height="24" class="mr-2" /></span>
            <span>Back To Main Library</span>
        </a>
    </div>

    <ProjectScroller
        filters={subnauticaOneFilters}
        maxProjects={60}
        loadingLabel="Loading Subnautica 1 mods..."
        emptyLabel="No Subnautica 1 mods have been imported yet."
        errorLabel="Subnautica 1 mods could not be loaded right now."
    />
</div>