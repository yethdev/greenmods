<script lang="ts">
    import { client } from "$lib/api";
    import { onMount } from "svelte";
    import { unwrapOrNull } from "@modhost/api";
    import type { ProjectCollection } from "@modhost/api";
    import CollectionCard from "./CollectionCard.svelte";

    interface Props {
        limit?: number;
        loadingLabel?: string;
        emptyLabel?: string;
        errorLabel?: string;
    }

    const {
        limit = 6,
        loadingLabel = "Loading featured collections...",
        emptyLabel = "No collections have been published yet.",
        errorLabel = "Collections could not be loaded right now.",
    }: Props = $props();

    let loading = $state(true);
    let loadFailed = $state(false);
    let collections = $state<ProjectCollection[]>([]);

    onMount(async () => {
        const data = unwrapOrNull(await client.collections().list());

        if (data) {
            collections = data.slice(0, limit);
        } else {
            loadFailed = true;
        }

        loading = false;
    });
</script>

<div aria-live="polite">
    {#if loading}
        <p class="text-sm opacity-70" role="status">{loadingLabel}</p>
    {:else if loadFailed}
        <p class="text-sm opacity-70">{errorLabel}</p>
    {:else if collections.length == 0}
        <p class="text-sm opacity-70">{emptyLabel}</p>
    {:else}
        <div class="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
            {#each collections as collection}
                <CollectionCard {collection} />
            {/each}
        </div>
    {/if}
</div>