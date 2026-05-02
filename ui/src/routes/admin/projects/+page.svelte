<script lang="ts">
    import { client } from "$/lib/api";
    import { capText } from "$/lib/util";
    import { goto } from "$app/navigation";
    import { showGenericDeleteModal } from "$components/modals/modals";
    import Icon from "@iconify/svelte";
    import { ErrorResponse } from "@modhost/api";
    import type { FullProject } from "@modhost/api";
    import { getModalStore, getToastStore } from "@skeletonlabs/skeleton";
    import { onMount } from "svelte";

    const modals = getModalStore();
    const toasts = getToastStore();

    let projects = $state<FullProject[]>([]);
    let loading = $state(true);

    const deleteProject = (project: FullProject) => {
        return (ev: MouseEvent) => {
            ev.preventDefault();
            ev.stopPropagation();

            showGenericDeleteModal(modals, {
                message: `Are you sure you want to delete the project, ${project.name}?`,
                callback: async () => {
                    try {
                        const res = await client.admin().project(project.id).delete();

                        if (res instanceof ErrorResponse) throw res;

                        await updateProjectList();
                    } catch (ex) {
                        toasts.trigger({
                            message: `Error: ${ex}`,
                            hideDismiss: true,
                            timeout: 5000,
                            background: "variant-filled-error",
                        });
                    }
                },
            });
        };
    };

    const updateProjectList = async () => {
        loading = true;

        const res = await client.admin().allProjects();

        if (res instanceof ErrorResponse) {
            toasts.trigger({
                message: `Error: ${res}`,
                hideDismiss: true,
                timeout: 5000,
                background: "variant-filled-error",
            });

            return;
        }

        projects = res.sort(
            (a, b) => new Date(a.updated_at).valueOf() - new Date(b.updated_at).valueOf(),
        );

        loading = false;
    };

    onMount(updateProjectList);
</script>

{#if loading}
    <div class="flex h-full w-full flex-row items-center justify-center py-16 text-xl">
        Loading...
    </div>
{:else}
    <div class="flex w-full flex-col items-start justify-start space-y-2">
        {#each projects as project}
            <a
                href="/p/{project.slug}"
                class="card hover:bg-surface-700 flex h-16 w-full cursor-pointer flex-row items-center justify-between p-4 transition-all"
            >
                <div class="flex h-full flex-row items-center justify-start space-x-3">
                    {#if project.authors[0].github_id == -1}
                        <img src="/modhost.png" alt="Author's avatar" class="h-full rounded-lg" />
                    {:else}
                        <img
                            src="https://avatars.githubusercontent.com/u/{project.authors[0]
                                .github_id}"
                            alt={project.authors[0].username}
                            class="h-full rounded-lg"
                        />
                    {/if}

                    <div>
                        <p class="flex flex-row items-center justify-start space-x-2 text-lg">
                            <span>{project.name}</span>
                            <span class="text-base text-gray-400"
                                >by {project.authors[0].username}</span
                            >
                            <span
                                class="variant-filled-secondary badge flex flex-row items-center justify-center space-x-1 px-[0.3rem] py-0"
                            >
                                {#if project.visibility == "Public"}
                                    <Icon icon="tabler:eye" height="18" />
                                {:else}
                                    <Icon icon="tabler:eye-off" height="18" />
                                {/if}
                                <span>{project.visibility}</span>
                            </span>
                        </p>

                        <p class="text-gray-400">{capText(project.description, 75)}</p>
                    </div>
                </div>
                <div class="flex h-full flex-row items-center justify-end space-x-3">
                    <button
                        class="text-success-500 hover:text-success-300 transition-all"
                        type="button"
                        onclick={(ev) => {
                            ev.preventDefault();
                            ev.stopPropagation();

                            goto(`/p/${project.slug}/edit`);
                        }}
                    >
                        <Icon icon="tabler:pencil" width="20" />
                    </button>

                    <button
                        class="text-error-500 hover:text-error-300 transition-all"
                        type="button"
                        onclick={deleteProject(project)}
                    >
                        <Icon icon="tabler:trash" width="20" />
                    </button>
                </div>
            </a>
        {/each}
    </div>
{/if}
