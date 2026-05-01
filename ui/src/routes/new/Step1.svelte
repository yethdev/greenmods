<script lang="ts">
    import BetterStep from "$components/ui/stepper/BetterStep.svelte";
    import { client } from "$lib/api";
    import { createSlug } from "$lib/util";
    import Icon from "@iconify/svelte";
    import { unwrapOrNull } from "@modhost/api";

    interface Props {
        name: string;
        slug: string;
        description: string;
    }

    let { name = $bindable(), slug = $bindable(), description = $bindable() }: Props = $props();
    let slugError = $state(false);
    const slugOk = $derived(/^[a-z0-9](?:[a-z0-9-]{1,62}[a-z0-9])$/.test(slug));
    const locked = $derived(
        name.trim().length < 3 || description.trim().length < 10 || !slugOk || slugError,
    );

    const updateSlug = async () => {
        slugError = false;
        slug = createSlug(name);
        slugError = !!unwrapOrNull(await client.project(slug).get());
    };
</script>

{#snippet header()}
    <p class="text-primary-500 mb-2 flex flex-row items-center justify-start">
        <Icon icon="tabler:info-circle" height="24" class="mr-2" />
        General Information
    </p>
{/snippet}

<BetterStep {header} {locked}>
    <div class="card variant-soft-primary w-full p-4">
        <p class="text-primary-500 mb-2 flex flex-row items-center justify-start">
            <Icon icon="tabler:tag" height="24" class="mr-2" />
            Display Name
        </p>

        <input
            type="text"
            placeholder="Example: My Package"
            class="input rounded-md"
            oninput={updateSlug}
            bind:value={name}
        />
    </div>

    <div class="card variant-soft-primary w-full p-4">
        <p class="text-primary-500 mb-2 flex flex-row items-center justify-start">
            <Icon icon="tabler:link" height="24" class="mr-2" />
            Slug
        </p>

        <input
            type="text"
            placeholder="Example: my-package"
            class="input rounded-md"
            bind:value={slug}
        />

        {#if slugError}
            <p class="text-error-500 ml-1 mt-2">Project already exists!</p>
        {:else if slug != "" && !slugOk}
            <p class="text-error-500 ml-1 mt-2">
                Use 3 to 64 lowercase letters, numbers, and dashes.
            </p>
        {/if}
    </div>

    <div class="card variant-soft-primary w-full p-4">
        <p class="text-primary-500 mb-2 flex flex-row items-center justify-start">
            <Icon icon="tabler:info-circle" height="24" class="mr-2" />
            Summary
        </p>

        <input
            type="text"
            placeholder="A short description of your project"
            class="input rounded-md"
            bind:value={description}
        />

        {#if description != "" && description.trim().length < 10}
            <p class="text-error-500 ml-1 mt-2">Use at least 10 characters.</p>
        {/if}
    </div>
</BetterStep>
