<script lang="ts">
    import BetterStep from "$components/ui/stepper/BetterStep.svelte";
    import Icon from "@iconify/svelte";
    import { Carta, MarkdownEditor } from "carta-md";

    interface Props {
        editor: Carta;
        readme: string;
    }

    let { editor, readme = $bindable() }: Props = $props();
    const locked = $derived(readme.trim().length < 20);
</script>

{#snippet header()}
    <p class="text-primary-500 mb-2 flex flex-row items-center justify-start">
        <Icon icon="tabler:book" height="24" class="mr-2" />
        Readme
    </p>
{/snippet}

<BetterStep {header} {locked}>
    <div class="card variant-soft-primary w-full p-4">
        <p class="text-primary-500 mb-2 flex flex-row items-center justify-start">
            <Icon icon="tabler:file-description" height="24" class="mr-2" />
            Description
        </p>

        {#if locked}
            <p class="text-error-500 mb-3 text-sm">Add at least 20 characters before creating.</p>
        {/if}

        <MarkdownEditor carta={editor} bind:value={readme} mode="tabs" />
    </div>
</BetterStep>
