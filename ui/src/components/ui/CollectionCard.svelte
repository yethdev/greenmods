<script lang="ts">
    import { base } from "$app/paths";
    import { formatDate } from "$lib/util";
    import type { ProjectCollection } from "@modhost/api";
    import Icon from "@iconify/svelte";

    interface Props {
        collection: ProjectCollection;
    }

    const { collection }: Props = $props();

    const totalDownloads = $derived(
        collection.projects.reduce((sum, project) => sum + project.downloads, 0),
    );
    const featuredProjects = $derived(collection.projects.slice(0, 3));
</script>

<a
    href={`${base}/collections/${collection.slug}`}
    class="card hover:variant-soft-primary flex h-full flex-col gap-4 p-4 transition-all"
>
    <div class="flex items-start justify-between gap-3">
        <div class="space-y-2">
            <div class="flex flex-wrap items-center gap-2">
                <span class="variant-filled-secondary badge uppercase tracking-[0.18em]"
                    >Collection</span
                >
                <span class="text-xs opacity-60">Updated {formatDate(new Date(collection.updated_at))}</span>
            </div>

            <div>
                <h3 class="text-lg font-bold">{collection.name}</h3>
                <p class="mt-1 text-sm opacity-70">{collection.description}</p>
            </div>
        </div>

        <span class="variant-filled-primary badge whitespace-nowrap">
            {collection.projects.length} {collection.projects.length == 1 ? "mod" : "mods"}
        </span>
    </div>

    <div class="grid gap-3 md:grid-cols-2">
        <div class="rounded-xl border border-white/10 p-3">
            <p class="text-xs uppercase tracking-[0.18em] opacity-50">Curated by</p>
            <div class="mt-2 flex items-center gap-3">
                {#if collection.owner.github_id == -1}
                    <img
                        src="/modhost.png"
                        alt="collection owner avatar"
                        class="h-10 w-10 rounded-lg object-cover"
                    />
                {:else}
                    <img
                        src={`https://avatars.githubusercontent.com/u/${collection.owner.github_id}`}
                        alt="collection owner avatar"
                        class="h-10 w-10 rounded-lg object-cover"
                    />
                {/if}

                <div>
                    <p class="font-semibold">{collection.owner.username}</p>
                    <p class="text-sm opacity-60">{totalDownloads} total downloads</p>
                </div>
            </div>
        </div>

        <div class="rounded-xl border border-white/10 p-3">
            <p class="text-xs uppercase tracking-[0.18em] opacity-50">Included mods</p>
            <div class="mt-2 flex flex-wrap gap-2">
                {#each featuredProjects as project}
                    <span class="variant-soft-secondary badge">{project.name}</span>
                {/each}

                {#if collection.projects.length > featuredProjects.length}
                    <span class="badge border border-white/10">
                        +{collection.projects.length - featuredProjects.length} more
                    </span>
                {/if}
            </div>
        </div>
    </div>

    <div class="flex items-center justify-between border-t border-white/10 pt-3 text-sm opacity-70">
        <span class="flex items-center gap-2">
            <Icon icon="tabler:stack-2" width="18" />
            Ordered collection with project notes
        </span>

        <span class="flex items-center gap-1 font-medium">
            Open collection
            <Icon icon="tabler:arrow-right" width="16" />
        </span>
    </div>
</a>