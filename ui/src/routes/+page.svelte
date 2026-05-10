<script lang="ts">
    import { _ } from "svelte-i18n";
    import CollectionRail from "$components/ui/CollectionRail.svelte";
    import ProjectScroller from "$components/ui/ProjectScroller.svelte";
    import { siteConfig } from "$lib/config";
    import { absoluteSiteUrl } from "$lib/seo";
    import Icon from "@iconify/svelte";
    import type { Facet } from "@modhost/api";

    const homeDescription =
        "Browse, publish, and sync Subnautica mods with GitHub releases, curated collections, and separate libraries for each game.";
    const mainLibraryFilters: Facet[] = [["exclude_tags", ["subnautica-1", "subnautica-below-zero"]]];
    const websiteJsonLd = JSON.stringify({
        "@context": "https://schema.org",
        "@type": "WebSite",
        name: siteConfig.siteName,
        url: siteConfig.siteUrl,
        description: homeDescription,
        about: [
            "open-source software",
            "project discovery",
            "compatibility tracking",
            "web platform",
        ],
        potentialAction: {
            "@type": "SearchAction",
            target: `${absoluteSiteUrl("/s")}?q={search_term_string}`,
            "query-input": "required name=search_term_string",
        },
    });

    const repoFiles = [
        ".openmods/install.json",
        ".openmods/FAQ.md",
        ".openmods/LINKS.md",
        "README.md",
    ];

    const featureCards = [
        {
            icon: "tabler:brand-github",
            title: "GitHub-native publishing",
            copy: "Link a public repo, keep metadata in the repo, and let releases ship version files automatically.",
        },
        {
            icon: "tabler:stack-2",
            title: "Curated collections",
            copy: "Publish ordered mod stacks with notes, compatibility picks, and direct download paths for full playthrough setups.",
        },
        {
            icon: "tabler:folders",
            title: "Separate game libraries",
            copy: "Keep Subnautica 1, Below Zero, and the main library easy to browse without flattening everything into one index.",
        },
    ];
</script>

<svelte:head>
    <title>{siteConfig.siteName} | GitHub-first mod hosting for Subnautica</title>
    <meta name="description" content={homeDescription} />
    <meta name="subject" content="Subnautica mod hosting" />
    <meta name="classification" content="Game mod platform" />
    <meta name="audience" content="Subnautica mod authors and players" />
    <meta property="og:title" content={`${siteConfig.siteName} | GitHub-first mod hosting for Subnautica`} />
    <meta property="og:description" content={homeDescription} />
    <meta name="twitter:title" content={`${siteConfig.siteName} | GitHub-first mod hosting for Subnautica`} />
    <meta name="twitter:description" content={homeDescription} />
    <script type="application/ld+json">{websiteJsonLd}</script>
</svelte:head>

<div
    class="from-primary-900 via-secondary-700 to-surface-900 m-0 flex w-full flex-col items-center justify-center bg-gradient-to-b from-10% to-90% px-8 py-32 md:px-10 md:py-36"
>
    <h1 class="flex flex-row items-center justify-center text-3xl md:text-4xl">
        <span>{siteConfig.siteName}</span>

        {#if siteConfig.showBeta}
            <span class="variant-filled-primary badge ml-4">{$_("site.beta")}</span>
        {/if}
    </h1>

    <span class="mt-3 text-center text-xl font-bold">{siteConfig.tagline}</span>
    <p class="mt-4 max-w-3xl text-center text-base opacity-80 md:text-lg">
        GitHub-first mod publishing, curated collections, and separate libraries for the games that
        actually need their own browsing lanes.
    </p>

    <div class="mt-12 flex flex-col items-center justify-center gap-4 md:flex-row md:flex-wrap">
        <a href="/s" class="variant-filled-primary btn">
            <span><Icon icon="tabler:search" height="24" class="mr-2" /></span>
            <span>{$_(`site.browse.${siteConfig.type}`)}</span>
        </a>

        <a href="/collections" class="variant-soft-secondary btn border border-white/10">
            <span><Icon icon="tabler:stack-2" height="24" class="mr-2" /></span>
            <span>Collections</span>
        </a>

        <a href="/one" class="variant-soft-secondary btn border border-white/10">
            <span><Icon icon="tabler:anchor" height="24" class="mr-2" /></span>
            <span>Subnautica 1 Library</span>
        </a>

        <a href="/zero" class="variant-soft-secondary btn border border-white/10">
            <span><Icon icon="tabler:snowflake" height="24" class="mr-2" /></span>
            <span>Below Zero Library</span>
        </a>
    </div>

    <div class="mt-14 grid w-full max-w-6xl gap-4 md:grid-cols-3">
        {#each featureCards as feature}
            <div class="rounded-2xl border border-white/10 bg-black/15 p-4 text-left backdrop-blur-sm">
                <div class="mb-3 flex items-center gap-3">
                    <span class="variant-filled-primary badge rounded-full p-2">
                        <Icon icon={feature.icon} width="18" />
                    </span>
                    <h2 class="text-lg font-semibold">{feature.title}</h2>
                </div>

                <p class="text-sm leading-6 opacity-80">{feature.copy}</p>
            </div>
        {/each}
    </div>
</div>

<section class="mx-auto w-full max-w-6xl px-4 py-10">
    <div class="card overflow-hidden p-0">
        <div class="grid gap-0 md:grid-cols-[1.1fr,0.9fr]">
            <div class="p-6 md:p-8">
                <span class="variant-filled-secondary badge uppercase tracking-[0.2em]">Push code, your mod ships</span>
                <h2 class="mt-4 text-3xl font-bold">Repository-owned metadata, not another form to babysit.</h2>
                <p class="mt-3 max-w-2xl opacity-80">
                    Point a project at GitHub and keep your release docs in the repo. README, FAQ,
                    links, and installer metadata can travel with the code instead of drifting out of sync.
                </p>

                <div class="mt-6 grid gap-3 md:grid-cols-3">
                    <div class="rounded-xl border border-white/10 p-4">
                        <p class="text-xs uppercase tracking-[0.18em] opacity-50">1. Link the repo</p>
                        <p class="mt-2 font-semibold">Use the project source field as the GitHub origin.</p>
                    </div>

                    <div class="rounded-xl border border-white/10 p-4">
                        <p class="text-xs uppercase tracking-[0.18em] opacity-50">2. Commit metadata</p>
                        <p class="mt-2 font-semibold">Keep install, FAQ, and links beside the actual mod source.</p>
                    </div>

                    <div class="rounded-xl border border-white/10 p-4">
                        <p class="text-xs uppercase tracking-[0.18em] opacity-50">3. Publish releases</p>
                        <p class="mt-2 font-semibold">Release assets become downloadable versions without a second upload step.</p>
                    </div>
                </div>
            </div>

            <div class="border-l border-white/10 bg-black/10 p-6 md:p-8">
                <p class="text-xs uppercase tracking-[0.18em] opacity-50">Tracked repo files</p>
                <div class="mt-4 flex flex-wrap gap-2">
                    {#each repoFiles as file}
                        <span class="variant-soft-secondary badge border border-white/10">{file}</span>
                    {/each}
                </div>

                <div class="mt-6 rounded-xl border border-white/10 p-4 text-sm leading-6 opacity-80">
                    Collections, GitHub sync, and separate game libraries are the core workflow now.
                    The site still keeps manual uploads, but you no longer have to duplicate repo metadata just to ship updates.
                </div>
            </div>
        </div>
    </div>
</section>

<section class="mx-auto w-full max-w-6xl px-4 py-4">
    <div class="mb-5 flex items-end justify-between gap-4">
        <div>
            <p class="text-xs uppercase tracking-[0.18em] opacity-50">Community picks</p>
            <h2 class="text-3xl font-bold">Featured collections</h2>
        </div>

        <a href="/collections" class="anchor flex items-center gap-2 no-underline opacity-80 transition-all hover:opacity-100">
            Browse all collections
            <Icon icon="tabler:arrow-right" width="18" />
        </a>
    </div>

    <CollectionRail />
</section>

<section class="mx-auto w-full max-w-6xl px-4 pb-16 pt-8">
    <div class="mb-5">
        <p class="text-xs uppercase tracking-[0.18em] opacity-50">Main library</p>
        <h2 class="text-3xl font-bold">Trending right now</h2>
        <p class="mt-2 max-w-2xl opacity-75">
            The busiest public mods in the main library, excluding the separate Subnautica 1 and Below Zero lanes.
        </p>
    </div>

    <ProjectScroller filters={mainLibraryFilters} />
</section>
