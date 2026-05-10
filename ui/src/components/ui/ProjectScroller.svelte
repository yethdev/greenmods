<script lang="ts">
    import { splitToRows } from "$lib/util";
    import { onMount } from "svelte";
    import ScrollingProject from "./ScrollingProject.svelte";
    import type { Facet, FullProject } from "@modhost/api";
    import { searchProjects } from "$lib/state";

    interface Props {
        filters?: Facet[];
        loadingLabel?: string;
        emptyLabel?: string;
        errorLabel?: string;
        maxProjects?: number;
    }

    const {
        filters = [],
        loadingLabel = "Loading current mods...",
        emptyLabel = "No public mods have been added yet.",
        errorLabel = "Current mods could not be loaded right now.",
        maxProjects = 30,
    }: Props = $props();

    const rows = 3;
    let rowElements: HTMLDivElement[] = $state([]);

    let loading = $state(true);
    let loadFailed = $state(false);
    let projects: FullProject[] = $state([]);
    let selected = $derived(splitToRows(projects, rows));
    const renderRows = $derived(selected.filter((items) => items.length > 0));

    onMount(async () => {
        const pkgs = await searchProjects(undefined, 1, 100, "downloads", "desc", filters);

        if (pkgs) {
            projects = pkgs.hits >= maxProjects ? pkgs.results.slice(0, maxProjects) : pkgs.results;
        } else {
            loadFailed = true;
        }

        loading = false;
    });

    const inHandler = (idx: number) => {
        return () => {
            const el = rowElements[idx];

            if (el) {
                el.style.animationPlayState = "paused";
            }
        };
    };

    const outHandler = (idx: number) => {
        return () => {
            const el = rowElements[idx];

            if (el) {
                el.style.animationPlayState = "running";
            }
        };
    };
</script>

<div class="mt-14 flex flex-col space-y-4" aria-live="polite">
    {#if loading}
        <p class="text-center text-sm opacity-70" role="status">{loadingLabel}</p>
    {:else if loadFailed}
        <p class="text-center text-sm opacity-70">{errorLabel}</p>
    {:else if projects.length == 0}
        <p class="text-center text-sm opacity-70">{emptyLabel}</p>
    {:else}
        {#each renderRows as items, index}
            <div
                class="hide-scrollbar flex w-screen select-none flex-row gap-6 overflow-hidden whitespace-nowrap"
            >
                <div
                    class="animate-scroll flex w-max whitespace-nowrap"
                    class:anim-reverse={!(index % 2 == 0)}
                    class:anim-mid={!(index % 2 == 0)}
                    bind:this={rowElements[index]}
                >
                    {#each [false, true] as duplicate}
                        <div class="flex min-w-full flex-shrink-0 gap-6 pr-6 whitespace-nowrap">
                            {#each items as pkg}
                                <ScrollingProject {index} {pkg} {inHandler} {outHandler} {duplicate} />
                            {/each}
                        </div>
                    {/each}
                </div>
            </div>
        {/each}
    {/if}
</div>
