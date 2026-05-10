<script lang="ts">
    import { getProjectPreviewImage } from "$lib/api";
    import { capText, getPrimaryProjectCreator } from "$lib/util";
    import type { FullProject } from "@modhost/api";
    import { onMount } from "svelte";
    import Icon from "@iconify/svelte";

    interface Props {
        pkg: FullProject;
        index: number;
        inHandler: (index: number) => () => void;
        outHandler: (index: number) => () => void;
        duplicate?: boolean;
    }

    const { pkg, index, inHandler, outHandler, duplicate = false }: Props = $props();

    let img = $state<string | undefined>(undefined);
    const creator = $derived(getPrimaryProjectCreator(pkg));

    onMount(async () => {
        img = await getProjectPreviewImage(pkg.id);
    });
</script>

<!-- svelte-ignore a11y_mouse_events_have_key_events -->
<a
    class="border-surface-500 bg-surface-700 hover:bg-surface-500 flex cursor-pointer flex-row gap-4 rounded-xl border-[1px] p-4 transition-all"
    href="/p/{pkg.slug}"
    aria-hidden={duplicate}
    tabindex={duplicate ? -1 : undefined}
    onmouseover={inHandler(index)}
    onmouseleave={outHandler(index)}
>
    {#if img}
        <img
            src={img}
            alt="package icon"
            class="my-auto mr-4 aspect-square h-12 rounded-lg"
            loading="lazy"
            decoding="async"
        />
    {:else if creator?.kind == "nexus"}
        <span
            class="bg-[#da8d32] text-black rounded-token my-auto mr-1 flex aspect-square h-10 items-center justify-center"
            aria-label="Nexus Mods"
        >
            <Icon icon="simple-icons:nexusmods" class="h-5 w-5" />
        </span>
    {:else if creator?.githubId == -1 || !creator}
        <img
            src="/modhost.png"
            alt="author's profile avatar"
            class="rounded-token my-auto mr-1 aspect-square h-10"
            loading="lazy"
            decoding="async"
        />
    {:else}
        <img
            src={`https://avatars.githubusercontent.com/u/${creator.githubId}`}
            alt="author's profile avatar"
            class="rounded-token my-auto mr-1 aspect-square h-10"
            loading="lazy"
            decoding="async"
            referrerpolicy="no-referrer"
        />
    {/if}
    <div class="project-info flex flex-col">
        <span class="title font-bold">
            {pkg.name}
        </span>
        <span class="description">
            {capText(pkg.description, 40)}
        </span>
    </div>
</a>
