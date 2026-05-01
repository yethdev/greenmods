<script lang="ts">
    import { _ } from "svelte-i18n";
    import { goto } from "$app/navigation";
    import { currentQuery } from "$lib/state";
    import ProjectScroller from "$components/ui/ProjectScroller.svelte";
    import { siteConfig } from "$lib/config";
    import { user } from "$lib/user";
    import Icon from "@iconify/svelte";

    let q = $state("");

    const runSearch = () => {
        const query = q.trim();
        $currentQuery = query;

        goto(query ? `/s?q=${encodeURIComponent(query)}` : "/s");
    };
</script>

<svelte:head>
    <title>{siteConfig.siteName}</title>
</svelte:head>

<section class="mx-auto flex min-h-full w-full max-w-screen-xl flex-col gap-12 px-4 py-10 md:px-8">
    <div class="grid items-center gap-8 lg:grid-cols-[1fr_22rem]">
        <div class="flex max-w-3xl flex-col gap-6">
            <div class="flex items-center gap-4">
                <img src="/favicon.png" alt="GreenMods logo" class="aspect-square w-14 rounded-lg" />
                <div>
                    <h1 class="text-4xl font-bold md:text-5xl">{siteConfig.siteName}</h1>
                    <p class="mt-1 text-lg text-surface-300">{siteConfig.tagline}</p>
                </div>
            </div>

            <form
                class="grid w-full grid-cols-[1fr_auto] gap-2"
                onsubmit={(ev) => {
                    ev.preventDefault();
                    runSearch();
                }}
            >
                <input
                    type="search"
                    class="input h-12 rounded-lg"
                    placeholder={$_(`search.placeholder.${siteConfig.type}`)}
                    bind:value={q}
                />
                <button type="submit" class="variant-filled-primary btn h-12 rounded-lg px-4">
                    <Icon icon="tabler:search" height="22" />
                    <span class="hidden sm:inline">Search</span>
                </button>
            </form>

            <div class="flex flex-wrap gap-3">
                <a href="/s" class="variant-soft-primary btn rounded-lg">
                    <Icon icon="tabler:list-search" height="22" />
                    {$_(`site.browse.${siteConfig.type}`)}
                </a>

                {#if $user}
                    <a href="/new" class="variant-soft-secondary btn rounded-lg">
                        <Icon icon="tabler:upload" height="22" />
                        Upload mod
                    </a>
                {/if}
            </div>
        </div>

        <div class="border-surface-700 bg-surface-800/60 rounded-lg border p-4">
            <dl class="grid grid-cols-2 gap-3 text-sm">
                <div>
                    <dt class="text-surface-400">Status tag</dt>
                    <dd class="mt-1 font-bold">Tested required</dd>
                </div>
                <div>
                    <dt class="text-surface-400">Versions</dt>
                    <dd class="mt-1 font-bold">Known builds</dd>
                </div>
                <div>
                    <dt class="text-surface-400">Loaders</dt>
                    <dd class="mt-1 font-bold">UE4SS, Paks</dd>
                </div>
                <div>
                    <dt class="text-surface-400">Game</dt>
                    <dd class="mt-1 font-bold">Subnautica 2</dd>
                </div>
            </dl>
        </div>
    </div>

    <ProjectScroller />
</section>
