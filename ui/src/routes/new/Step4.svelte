<script lang="ts">
    import BetterStep from "$components/ui/stepper/BetterStep.svelte";
    import { tags as allTags } from "$lib/meta";
    import Icon from "@iconify/svelte";

    interface Props {
        selectedTags: string[];
    }

    let { selectedTags = $bindable() }: Props = $props();
    const hasTested = $derived(selectedTags.includes("tested"));
    const locked = $derived(!hasTested || selectedTags.length < 1);

    const toggleTag = (id: string) => {
        if (selectedTags.includes(id)) {
            selectedTags = selectedTags.filter((tag) => tag !== id);
        } else {
            selectedTags = [...selectedTags, id];
        }
    };
</script>

{#snippet header()}
    <p class="text-primary-500 mb-2 flex flex-row items-center justify-start">
        <Icon icon="tabler:tags" height="24" class="mr-2" />
        Tags
    </p>
{/snippet}

<BetterStep {header} {locked}>
    <div class="card variant-soft-primary w-full p-4">
        <p class="text-primary-500 mb-2 flex flex-row items-center justify-start">
            <Icon icon="tabler:shield-check" height="24" class="mr-2" />
            Required status
        </p>

        <p class="mb-4 text-sm opacity-70">
            Select Tested after you have checked the mod against a known Subnautica 2 build.
        </p>

        {#if !hasTested}
            <p class="text-error-500 mb-3 text-sm">The Tested tag is required.</p>
        {/if}

        <div class="flex flex-wrap gap-2">
            {#each $allTags as tag}
                <button
                    type="button"
                    class="chip text-sm !outline-none {selectedTags.includes(tag.id)
                        ? 'variant-filled-primary'
                        : 'variant-soft-primary'}"
                    onclick={() => toggleTag(tag.id)}
                >
                    <Icon icon={tag.icon} width="18" class="mr-2" />
                    {tag.name}
                </button>
            {/each}
        </div>
    </div>
</BetterStep>
