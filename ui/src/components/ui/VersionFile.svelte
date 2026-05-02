<script lang="ts">
    import { _ } from "svelte-i18n";
    import { downloadFile, formatBytes, formatDate } from "$lib/util";
    import Icon from "@iconify/svelte";
    import type { ProjectFile, ProjectVersion } from "@modhost/api";
    import { apiBase } from "$lib/api";
    import { getToastStore } from "@skeletonlabs/skeleton";

    interface Props {
        version: ProjectVersion;
        file: ProjectFile;
        pkg: string;
    }

    const { version, file, pkg }: Props = $props();
    const toasts = getToastStore();

    let downloading = $state(false);
    let modOnlyDownloading = $state(false);
    let done = $state(false);
    let modOnlyDone = $state(false);
    let doneTimeout: number | undefined;
    let modOnlyDoneTimeout: number | undefined;
    const canModOnly = $derived(file.file_name.toLowerCase().endsWith(".zip"));

    const directDownload = async (ev: Event) => {
        ev.preventDefault();
        ev.stopPropagation();

        downloading = true;

        try {
            await downloadFile(
                `${apiBase}/projects/${pkg}/versions/${version.id}/download/${file.id}`,
                file.file_name,
            );

            done = true;

            if (doneTimeout) clearTimeout(doneTimeout);

            doneTimeout = setTimeout(() => {
                done = false;
            }, 1000) as any;
        } catch (err) {
            toasts.trigger({
                message: (err as Error).message || "Download failed.",
                hideDismiss: true,
                timeout: 5000,
                background: "variant-filled-error",
            });
        } finally {
            downloading = false;
        }
    };

    const modOnlyDownload = async (ev: Event) => {
        ev.preventDefault();
        ev.stopPropagation();

        modOnlyDownloading = true;

        try {
            await downloadFile(
                `${apiBase}/projects/${pkg}/versions/${version.id}/download/${file.id}/mod-only`,
            );

            modOnlyDone = true;

            if (modOnlyDoneTimeout) clearTimeout(modOnlyDoneTimeout);

            modOnlyDoneTimeout = setTimeout(() => {
                modOnlyDone = false;
            }, 1000) as any;
        } catch (err) {
            toasts.trigger({
                message: (err as Error).message || "Mod only download failed.",
                hideDismiss: true,
                timeout: 5000,
                background: "variant-filled-error",
            });
        } finally {
            modOnlyDownloading = false;
        }
    };
</script>

<a
    href="/p/{pkg}/versions/{version.id}"
    class="rounded-container-token hover:variant-soft-primary flex w-full items-center gap-2 p-2 text-left transition-all"
>
    <button
        type="button"
        class="variant-filled-secondary btn hover:variant-outline-primary p-2 transition-all"
        onclick={directDownload}
    >
        {#if done}
            <Icon icon="tabler:check" height="24" />
        {:else if downloading}
            <Icon icon="tabler:loader-2" height="24" class="animate-spin" />
        {:else}
            <Icon icon="tabler:download" height="24" />
        {/if}
    </button>

    {#if canModOnly}
        <button
            type="button"
            class="variant-soft-primary btn hover:variant-filled-primary gap-2 p-2 transition-all"
            onclick={modOnlyDownload}
            title="Download only the single mod file from this ZIP"
        >
            {#if modOnlyDone}
                <Icon icon="tabler:check" height="22" />
            {:else if modOnlyDownloading}
                <Icon icon="tabler:loader-2" height="22" class="animate-spin" />
            {:else}
                <Icon icon="tabler:file-download" height="22" />
            {/if}
            <span class="hidden sm:inline">Mod only</span>
        </button>
    {/if}

    <span class="ml-1 flex-auto">
        <dt class="select-text font-bold">{file.file_name}</dt>
        <dd class="text-sm opacity-50">
            {formatBytes(file.size)} &bull; Uploaded {formatDate(new Date(file.uploaded_at))}
        </dd>
    </span>
</a>
