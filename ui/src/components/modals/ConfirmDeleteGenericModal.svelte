<script lang="ts">
    import { _ } from "svelte-i18n";
    import { getModalStore, getToastStore } from "@skeletonlabs/skeleton";
    import { onMount } from "svelte";
    import type { ModalCallback } from "./modals";

    const modals = getModalStore();
    const toasts = getToastStore();
    let loading = $state(false);

    let callback = $state<ModalCallback>();
    let afterClose = $state<ModalCallback>();
    let message = $state<string>();

    onMount(() => {
        if (
            !$modals[0].meta ||
            !("callback" in $modals[0].meta) ||
            !("message" in $modals[0].meta)
        ) {
            toasts.trigger({
                message: `Error: Incorrectly set up generic deletion modal! This is a bug!`,
                hideDismiss: true,
                timeout: 5000,
                background: "variant-filled-error",
            });

            modals.close();

            return;
        }

        message = $modals[0].meta.message;
        callback = $modals[0].meta.callback;

        if ("afterClose" in $modals[0].meta) afterClose = $modals[0].meta.afterClose;
    });

    const confirmDelete = async () => {
        if (!callback) {
            toasts.trigger({
                message: `Error: Modal callback was undefined! This is a bug!`,
                hideDismiss: true,
                timeout: 5000,
                background: "variant-filled-error",
            });

            modals.close();

            return;
        }

        loading = true;

        callback();

        loading = false;
        modals.close();

        afterClose?.();

        message = undefined;
        callback = undefined;
        afterClose = undefined;
    };
</script>

{#if $modals[0]}
    <div class="w-modal-slim bg-surface-500 relative rounded-lg p-8 shadow-xl">
        <header class="text-2xl font-bold">Confirm Deletion</header>

        <p>{message}</p>

        <footer class="modal-footer mt-4 flex flex-row items-center">
            <button
                class="variant-filled-error btn hover:variant-ghost-error mr-2 !outline-none transition-all"
                disabled={loading}
                onclick={confirmDelete}>Delete</button
            >

            <button
                class="variant-filled-secondary btn hover:variant-ghost-primary mr-2 !outline-none transition-all"
                disabled={loading}
                onclick={() => modals.close()}>{$_("action.cancel")}</button
            >
        </footer>
    </div>
{/if}
