<script lang="ts">
    import { _ } from "svelte-i18n";
    import { base } from "$app/paths";
    import { page } from "$app/state";
    import { getProjectPreviewImage } from "$lib/api";
    import { formatDate, getPrimaryProjectCreator } from "$lib/util";
    import { onMount } from "svelte";
    import type { FullProject } from "@modhost/api";
    import Icon from "@iconify/svelte";

    interface Props {
        pkg: FullProject;
        customHeight?: number;
        compact?: boolean;
        showAvatar?: boolean;
        showDetails?: boolean;
        showName?: boolean;
        select?: (id: number) => void | Promise<void>;
    }

    const { pkg, customHeight, compact, showAvatar, showDetails, showName, select }: Props =
        $props();

    let img = $state<string | undefined>(undefined);
    const creator = $derived(getPrimaryProjectCreator(pkg));

    onMount(async () => {
        img = await getProjectPreviewImage(pkg.id);
    });
</script>

<a
    href={`${base}/p/${pkg.slug}`}
    class="card hover:variant-soft-primary flex p-4"
    class:flex-col={compact}
    onclick={() => select?.(pkg.id)}
    class:!variant-filled-primary={page.url.searchParams.get("id") == pkg.name}
    style={customHeight != null ? `height: ${customHeight}rem` : ""}
>
    {#if showAvatar && !compact}
        {#if img}
            <img
                src={img}
                alt="package icon"
                class="my-auto mr-4 aspect-square h-16 rounded-lg"
                loading="lazy"
                decoding="async"
            />
        {:else if creator?.kind == "nexus"}
            <span
                class="bg-[#da8d32] text-black rounded-token my-auto mr-4 flex aspect-square h-8 items-center justify-center"
                aria-label="Nexus Mods"
            >
                <Icon icon="simple-icons:nexusmods" class="h-4 w-4" />
            </span>
        {:else if creator?.githubId == -1 || !creator}
            <img
                src="/modhost.png"
                alt="author's profile avatar"
                class="rounded-token my-auto mr-4 aspect-square h-8"
                loading="lazy"
                decoding="async"
            />
        {:else}
            <img
                src={`https://avatars.githubusercontent.com/u/${creator.githubId}`}
                alt="author's profile avatar"
                class="rounded-token my-auto mr-4 aspect-square h-8"
                loading="lazy"
                decoding="async"
                referrerpolicy="no-referrer"
            />
        {/if}
    {/if}
    <dl class="my-auto">
        <dt class="mb-1 select-text font-bold">{pkg.name}</dt>
        <dd class="text-sm opacity-50">
            {#if showName}
                {$_("list.by")} <span class="select-text">{creator?.name ?? "Unknown"}</span>
            {/if}
        </dd>
        <dd class="text-sm opacity-50">
            <span
                >{pkg.downloads}
                {pkg.downloads == 1
                    ? $_("list.download_singluar")
                    : $_("list.download_plural")}</span
            >
        </dd>
        {#if showDetails}
            <dd class="text-sm opacity-50">
                {$_("list.published")}
                <span class="select-text">{formatDate(new Date(pkg.created_at))}</span>
            </dd>
            <dd class="text-sm opacity-50">
                {$_("list.updated")}
                <span class="select-text">{formatDate(new Date(pkg.updated_at))}</span>
            </dd>
        {/if}
    </dl>
</a>
