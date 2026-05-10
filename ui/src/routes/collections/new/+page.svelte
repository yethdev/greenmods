<script lang="ts">
    import { goto } from "$app/navigation";
    import { client } from "$lib/api";
    import { siteConfig } from "$lib/config";
    import { user } from "$lib/user";
    import { getToastStore } from "@skeletonlabs/skeleton";
    import { unwrapOrNull } from "@modhost/api";
    import type { ProjectVisibility } from "@modhost/api";
    import Icon from "@iconify/svelte";

    const toasts = getToastStore();

    let slug = $state("");
    let name = $state("");
    let description = $state("");
    let readme = $state("");
    let projectRefs = $state("");
    let visibility = $state<ProjectVisibility>("Public");
    let saving = $state(false);

    const createCollection = async () => {
        if (!$user) {
            toasts.trigger({
                message: "Sign in first to publish a collection.",
                hideDismiss: true,
                timeout: 4000,
                background: "variant-filled-error",
            });
            return;
        }

        saving = true;

        const collection = unwrapOrNull(
            await client.collections().create({
                slug,
                name,
                description,
                readme,
                visibility,
                projects: projectRefs
                    .split(/\r?\n|,/) 
                    .map((item) => item.trim())
                    .filter(Boolean),
            }),
        );

        saving = false;

        if (!collection) {
            toasts.trigger({
                message: "Collection could not be created. Check the fields and try again.",
                hideDismiss: true,
                timeout: 5000,
                background: "variant-filled-error",
            });
            return;
        }

        goto(`/collections/${collection.slug}`);
    };
</script>

<svelte:head>
    <title>Create Collection | {siteConfig.siteName}</title>
</svelte:head>

<div class="mx-auto flex w-full max-w-5xl flex-col gap-4">
    <div class="card p-6">
        <span class="variant-filled-secondary badge uppercase tracking-[0.2em]">Collections</span>
        <h1 class="mt-4 text-3xl font-bold">Publish a curated mod stack.</h1>
        <p class="mt-2 max-w-3xl opacity-80">
            Add project ids or slugs in order, write a short guide, and publish a collection page that
            players can install from top to bottom.
        </p>
    </div>

    <div class="grid gap-4 lg:grid-cols-[minmax(0,1fr),20rem]">
        <div class="flex min-w-0 flex-col gap-4">
            <div class="card p-4">
                <p class="text-primary-500 mb-2 flex items-center gap-2">
                    <Icon icon="tabler:link" width="20" />
                    Slug
                </p>
                <input class="input rounded-md" bind:value={slug} placeholder="example-run" />
            </div>

            <div class="card p-4">
                <p class="text-primary-500 mb-2 flex items-center gap-2">
                    <Icon icon="tabler:stack-2" width="20" />
                    Collection Name
                </p>
                <input class="input rounded-md" bind:value={name} placeholder="Example progression pack" />
            </div>

            <div class="card p-4">
                <p class="text-primary-500 mb-2 flex items-center gap-2">
                    <Icon icon="tabler:info-circle" width="20" />
                    Summary
                </p>
                <textarea
                    class="textarea min-h-28 rounded-md"
                    bind:value={description}
                    placeholder="What kind of run is this collection for?"
                ></textarea>
            </div>

            <div class="card p-4">
                <p class="text-primary-500 mb-2 flex items-center gap-2">
                    <Icon icon="tabler:list-numbers" width="20" />
                    Project Order
                </p>
                <textarea
                    class="textarea min-h-40 rounded-md font-mono"
                    bind:value={projectRefs}
                    placeholder="one project slug or id per line\nbase-library\nui-tweaks\nlate-game-pack"
                ></textarea>
                <p class="mt-2 text-sm opacity-70">Use project slugs or numeric ids. The order here becomes the install order.</p>
            </div>

            <div class="card p-4">
                <p class="text-primary-500 mb-2 flex items-center gap-2">
                    <Icon icon="tabler:file-description" width="20" />
                    Collection Notes
                </p>
                <textarea
                    class="textarea min-h-72 rounded-md"
                    bind:value={readme}
                    placeholder="# Install order\n\nExplain why each mod is in the stack, what to install first, and what players should know before they start."
                ></textarea>
            </div>
        </div>

        <aside class="card h-fit p-4">
            <p class="text-sm uppercase tracking-[0.18em] opacity-50">Publishing</p>

            <div class="mt-4 space-y-4">
                <div>
                    <label class="mb-2 block text-sm opacity-60" for="collection-visibility">Visibility</label>
                    <select id="collection-visibility" class="select rounded-lg !outline-none" bind:value={visibility}>
                        <option value="Public">Public</option>
                        <option value="Unlisted">Unlisted</option>
                        <option value="Private">Private</option>
                    </select>
                </div>

                <div class="rounded-xl border border-white/10 p-4 text-sm leading-6 opacity-80">
                    Collections are best for compatibility packs, streamer setups, challenge runs, and
                    progression routes where installation order matters.
                </div>

                <button
                    type="button"
                    class="variant-filled-primary btn w-full"
                    onclick={createCollection}
                    disabled={saving}
                >
                    <Icon icon="tabler:device-floppy" width="20" class="mr-2" />
                    {saving ? "Publishing..." : "Publish Collection"}
                </button>

                <a href="/collections" class="variant-soft-secondary btn w-full border border-white/10">
                    Cancel
                </a>
            </div>
        </aside>
    </div>
</div>