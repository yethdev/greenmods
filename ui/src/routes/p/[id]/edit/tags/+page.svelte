<script lang="ts">
    import { page } from "$app/stores";
    import { onMount } from "svelte";
    import { currentProject, editSaving } from "$lib/state";
    import { tags as allTags } from "$lib/meta";
    import Icon from "@iconify/svelte";
    import { getToastStore } from "@skeletonlabs/skeleton";
    import { client } from "$lib/api";
    import { ErrorResponse, unwrap } from "@modhost/api";

    const id = $derived($page.params.id);
    const toasts = getToastStore();

    let selectedTags = $state<string[]>([]);
    const locked = $derived(selectedTags.length < 1);

    onMount(() => {
        selectedTags = $currentProject?.tags ?? [];
    });

    const toggleTag = (tagId: string) => {
        if (selectedTags.includes(tagId)) {
            selectedTags = selectedTags.filter((tag) => tag !== tagId);
        } else {
            selectedTags = [...selectedTags, tagId];
        }
    };

    const save = async () => {
        if (locked) {
            toasts.trigger({
                message: "Choose at least one tag.",
                hideDismiss: true,
                timeout: 5000,
                background: "variant-filled-error",
            });

            return;
        }

        $editSaving = true;

        const res = await client.project(id).update({ tags: selectedTags });

        if (res instanceof ErrorResponse) {
            $editSaving = false;

            toasts.trigger({
                message: `Error saving tags: ${res.cause}`,
                hideDismiss: true,
                timeout: 5000,
                background: "variant-filled-error",
            });

            return;
        }

        $currentProject = unwrap(await client.project(id).get());
        selectedTags = $currentProject.tags ?? [];
        $editSaving = false;
    };
</script>

<p class="text-primary-500 mb-2 flex flex-row items-center justify-start">
    <Icon icon="tabler:tags" height="24" class="mr-2" />
    Manage Tags
</p>

<div class="card variant-soft-primary w-full p-4">
    <p class="text-primary-500 mb-2 flex flex-row items-center justify-start">
        <Icon icon="tabler:tags" height="24" class="mr-2" />
        Tags
    </p>

    {#if locked}
        <p class="text-error-500 mb-3 text-sm">Choose at least one tag before saving.</p>
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

<button
    type="button"
    class="variant-filled-primary btn hover:variant-ghost-primary hover:text-token mt-2 flex flex-row items-center justify-center rounded-lg transition-all"
    onclick={save}
    disabled={locked || $editSaving}
>
    <Icon icon="tabler:device-floppy" height="24" class="mr-2" />
    Save
</button>
