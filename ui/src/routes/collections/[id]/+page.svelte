<script lang="ts">
    import { page } from "$app/stores";
    import PackageList from "$components/ui/PackageList.svelte";
    import { client } from "$lib/api";
    import { siteConfig } from "$lib/config";
    import { absoluteSiteUrl, trimDescription } from "$lib/seo";
    import type { LoadingState } from "$lib/types";
    import { formatDate, markdown } from "$lib/util";
    import { onMount } from "svelte";
    import { unwrapOrNull } from "@modhost/api";
    import type { ProjectCollection } from "@modhost/api";
    import Icon from "@iconify/svelte";

    const id = $derived($page.params.id);

    let loadingState = $state<LoadingState>("loading");
    let collection = $state<ProjectCollection | null>(null);

    const pageTitle = $derived(
        collection ? `${collection.name} | ${siteConfig.siteName}` : `Collection | ${siteConfig.siteName}`,
    );
    const pageDescription = $derived(
        trimDescription(
            collection?.description,
            "Curated greenmods collection with ordered projects and install notes.",
        ),
    );
    const pageUrl = $derived(absoluteSiteUrl(`/collections/${id}`));
    const jsonLd = $derived(
        !collection
            ? ""
            : JSON.stringify({
                  "@context": "https://schema.org",
                  "@type": "CollectionPage",
                  name: collection.name,
                  url: pageUrl,
                  description: pageDescription,
                  isPartOf: siteConfig.siteUrl,
              }),
    );

    onMount(async () => {
        collection = unwrapOrNull(await client.collection(id).get());
        loadingState = collection ? "ready" : "failed";
    });
</script>

<svelte:head>
    <title>{pageTitle}</title>
    <meta name="description" content={pageDescription} />
    <meta property="og:title" content={pageTitle} />
    <meta property="og:description" content={pageDescription} />
    <meta property="og:url" content={pageUrl} />
    <meta name="twitter:title" content={pageTitle} />
    <meta name="twitter:description" content={pageDescription} />
    {#if jsonLd}
        <script type="application/ld+json">{jsonLd}</script>
    {/if}
</svelte:head>

{#if loadingState == "loading"}
    <div class="placeholder m-2 mx-auto w-32 animate-pulse"></div>
{:else if loadingState == "ready" && collection}
    <div class="grid w-full gap-4 md:grid-cols-[20rem,minmax(0,1fr)]">
        <aside class="card flex flex-col gap-4 p-4">
            <div class="space-y-2">
                <span class="variant-filled-secondary badge uppercase tracking-[0.2em]">Collection</span>
                <h1 class="text-3xl font-bold">{collection.name}</h1>
                <p class="opacity-75">{collection.description}</p>
            </div>

            <div class="rounded-xl border border-white/10 p-3">
                <p class="text-xs uppercase tracking-[0.18em] opacity-50">Curated by</p>
                <div class="mt-2 flex items-center gap-3">
                    {#if collection.owner.github_id == -1}
                        <img src="/modhost.png" alt="owner avatar" class="h-10 w-10 rounded-lg object-cover" />
                    {:else}
                        <img
                            src={`https://avatars.githubusercontent.com/u/${collection.owner.github_id}`}
                            alt="owner avatar"
                            class="h-10 w-10 rounded-lg object-cover"
                        />
                    {/if}

                    <div>
                        <a class="font-semibold hover:underline" href={`/u/${collection.owner.username}`}
                            >{collection.owner.username}</a
                        >
                        <p class="text-sm opacity-60">Updated {formatDate(new Date(collection.updated_at))}</p>
                    </div>
                </div>
            </div>

            <div class="grid grid-cols-2 gap-3">
                <div class="rounded-xl border border-white/10 p-3">
                    <p class="text-xs uppercase tracking-[0.18em] opacity-50">Projects</p>
                    <p class="mt-2 text-2xl font-bold">{collection.projects.length}</p>
                </div>

                <div class="rounded-xl border border-white/10 p-3">
                    <p class="text-xs uppercase tracking-[0.18em] opacity-50">Downloads</p>
                    <p class="mt-2 text-2xl font-bold">
                        {collection.projects.reduce((sum, project) => sum + project.downloads, 0)}
                    </p>
                </div>
            </div>

            <div class="flex flex-wrap gap-2">
                <a href="/collections" class="variant-soft-secondary btn border border-white/10">
                    <Icon icon="tabler:layout-grid" width="20" class="mr-2" />
                    All Collections
                </a>

                <a href="/s" class="variant-filled-primary btn">
                    <Icon icon="tabler:search" width="20" class="mr-2" />
                    Browse Mods
                </a>
            </div>
        </aside>

        <div class="flex min-w-0 flex-col gap-4">
            <section class="card p-4">
                <dt class="mb-2 text-sm opacity-50">Collection Notes</dt>
                <dd class="style-markdown flex flex-col overflow-x-auto *:select-text">
                    {@html markdown(collection.readme)}
                </dd>
            </section>

            <section class="card p-4">
                <div class="mb-4 flex items-center justify-between gap-3">
                    <div>
                        <p class="text-sm uppercase tracking-[0.18em] opacity-50">Included Mods</p>
                        <h2 class="text-2xl font-bold">Ordered project list</h2>
                    </div>

                    <span class="variant-filled-primary badge">{collection.projects.length} entries</span>
                </div>

                <div class="grid grid-cols-1 gap-2 lg:grid-cols-2">
                    <PackageList
                        showDetails
                        packages={{
                            results: collection.projects,
                            hits: collection.projects.length,
                            page: 1,
                            pages: 1,
                            total: collection.projects.length,
                        }}
                    />
                </div>
            </section>
        </div>
    </div>
{:else}
    <div class="card p-6">
        <p class="text-lg font-semibold">Collection not found.</p>
        <p class="mt-2 opacity-70">The requested collection either does not exist or is not public.</p>
    </div>
{/if}