<script lang="ts">
    import { client } from "$/lib/api";
    import { showGenericDeleteModal } from "$components/modals/modals";
    import Icon from "@iconify/svelte";
    import { ErrorResponse } from "@modhost/api";
    import type { User } from "@modhost/api";
    import { getModalStore, getToastStore } from "@skeletonlabs/skeleton";
    import { onMount } from "svelte";

    const modals = getModalStore();
    const toasts = getToastStore();

    let users = $state<User[]>([]);
    let loading = $state(true);

    const deleteUser = (user: User) => {
        return (ev: MouseEvent) => {
            ev.preventDefault();
            ev.stopPropagation();

            showGenericDeleteModal(modals, {
                // ¿Estás seguro que quieres que borrar este usuario?
                // - me, after 4 years of spanish classes
                message: `Are you sure you want to delete the user, ${user.username}?`,
                callback: async () => {
                    try {
                        const res = await client.admin().user(user.id).delete();

                        if (res instanceof ErrorResponse) throw res;

                        await updateUserList();
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

    const updateUserList = async () => {
        loading = true;

        const res = await client.admin().allUsers();

        if (res instanceof ErrorResponse) {
            toasts.trigger({
                message: `Error: ${res}`,
                hideDismiss: true,
                timeout: 5000,
                background: "variant-filled-error",
            });

            return;
        }

        users = res;
        loading = false;
    };

    onMount(updateUserList);
</script>

{#if loading}
    <div class="flex h-full w-full flex-row items-center justify-center py-16 text-xl">
        Loading...
    </div>
{:else}
    <div class="flex w-full flex-col items-start justify-start space-y-2">
        {#each users as user}
            <a
                href="/u/{user.username}"
                class="card hover:bg-surface-700 flex h-16 w-full cursor-pointer flex-row items-center justify-between p-4 transition-all"
            >
                <div class="flex h-full flex-row items-center justify-start space-x-3">
                    <img
                        src="https://avatars.githubusercontent.com/u/{user.github_id}"
                        alt={user.username}
                        class="h-full rounded-lg"
                    />

                    <p class="text-lg">{user.username}</p>
                </div>
                <div class="flex h-full flex-row items-center justify-end space-x-3">
                    <button
                        class="text-error-500 hover:text-error-300 transition-all"
                        type="button"
                        onclick={deleteUser(user)}
                    >
                        <Icon icon="tabler:trash" width="20" />
                    </button>
                </div>
            </a>
        {/each}
    </div>
{/if}
