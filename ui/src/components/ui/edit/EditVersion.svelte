<script lang="ts">
    import { _ } from "svelte-i18n";
    import { getModalStore } from "@skeletonlabs/skeleton";
    import { formatDate, downloadFile } from "$lib/util";
    import { currentProject } from "$lib/state";
    import Icon from "@iconify/svelte";
    import type { ProjectVersion } from "@modhost/api";
    import { isNexusSource } from "$lib/util";

    interface Props {
        version: ProjectVersion;
        pkg: string;
    }

    const { version, pkg }: Props = $props();
    const modals = getModalStore();

    let downloading = $state(false);
    let done = $state(false);
    const hasFiles = $derived(version.files.length > 0);
    const externalHref = $derived(isNexusSource($currentProject?.source) ? $currentProject?.source : null);

    const handleDelete = async (ev: Event) => {
        ev.preventDefault();
        ev.stopPropagation();

        modals.trigger({
            type: "component",
            component: "confirmDeleteVersion",
            meta: { versionId: version.id },
        });
    };

    let doneTimeout: ReturnType<typeof setTimeout> | undefined;

    const directDownload = async (ev: Event) => {
        ev.preventDefault();
        ev.stopPropagation();

        if (!hasFiles) {
            if (externalHref) {
                globalThis.open(externalHref, "_blank", "noopener,noreferrer");
            }

            return;
        }

        downloading = true;

        const file = version.files[0];

        await downloadFile(
            `/api/v1/projects/${pkg}/versions/${version.id}/download/${file.id}`,
            file.file_name,
        );

        downloading = false;
        done = true;

        if (doneTimeout) clearTimeout(doneTimeout);

        doneTimeout = setTimeout(() => {
            done = false;
        }, 1000);
    };
</script>

<a
    href="/p/{pkg}/edit/versions/edit/{version.id}"
    class="rounded-container-token hover:variant-soft-primary flex w-full items-center gap-2 p-2 text-left transition-all"
>
    <button
        type="button"
        class="variant-filled-secondary btn hover:variant-outline-primary p-2 transition-all"
        onclick={directDownload}
        title={hasFiles ? "Download" : externalHref ? "Open on Nexus Mods" : "No downloadable file"}
    >
        {#if done}
            <Icon icon="tabler:check" height="24" />
        {:else if downloading}
            <Icon icon="tabler:loader-2" height="24" class="animate-spin" />
        {:else if !hasFiles && externalHref}
            <Icon icon="tabler:external-link" height="24" />
        {:else}
            <Icon icon="tabler:download" height="24" />
        {/if}
    </button>

    <span class="ml-1 flex-auto">
        <dt class="select-text font-bold">{version.name}</dt>
        <dd class="text-sm opacity-50">{formatDate(new Date(version.created_at))}</dd>
    </span>

    <!-- This has no onclick handler because it just passes through to the underlying link -->
    <button
        class="variant-glass-primary btn btn-sm hover:variant-filled-primary transition-all"
        type="button"
    >
        <Icon icon="tabler:pencil" height="24" />
    </button>

    <button
        class="variant-glass-error btn btn-sm hover:variant-filled-error transition-all"
        type="button"
        onclick={handleDelete}
    >
        <Icon icon="tabler:trash" height="24" />
    </button>
</a>
