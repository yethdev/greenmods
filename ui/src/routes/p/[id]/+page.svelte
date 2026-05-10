<script lang="ts">
    import { _ } from "svelte-i18n";
    import { formatDate, markdown } from "$lib/util";
    import { currentProject } from "$lib/state";
    import Icon from "@iconify/svelte";

    const installManifest = $derived.by(() => {
        const raw = $currentProject?.install_json;

        if (!raw) return "";

        try {
            return JSON.stringify(JSON.parse(raw), null, 2);
        } catch {
            return raw;
        }
    });
</script>

<div class="flex w-full flex-col gap-4">
    <section class="card h-fit w-full p-4">
        <dt class="mb-2 text-sm opacity-50">Description</dt>
        <dd
            class="style-markdown flex select-text flex-col items-start overflow-x-scroll *:select-text"
        >
            {@html markdown($currentProject?.readme ?? "")}
        </dd>
    </section>

    {#if $currentProject?.repo_sync || $currentProject?.faq || $currentProject?.repo_links || installManifest}
        <div class="grid w-full gap-4 xl:grid-cols-2">
            {#if $currentProject?.repo_sync}
                <section class="card h-fit p-4">
                    <div class="mb-4 flex items-start justify-between gap-3">
                        <div>
                            <dt class="text-sm opacity-50">Repository Automation</dt>
                            <h2 class="text-xl font-bold">GitHub sync is active</h2>
                        </div>

                        <span class="variant-filled-secondary badge">GitHub</span>
                    </div>

                    <div class="space-y-3 text-sm opacity-85">
                        <p class="flex items-center gap-2">
                            <Icon icon="tabler:brand-github" width="18" />
                            {$currentProject.repo_sync.repo_owner}/{$currentProject.repo_sync.repo_name}
                        </p>

                        {#if $currentProject.repo_sync.default_branch}
                            <p class="flex items-center gap-2">
                                <Icon icon="tabler:git-branch" width="18" />
                                Default branch: {$currentProject.repo_sync.default_branch}
                            </p>
                        {/if}

                        <div class="flex flex-wrap gap-2">
                            {#if $currentProject.repo_sync.sync_readme}
                                <span class="variant-filled-primary badge">README sync</span>
                            {/if}
                            {#if $currentProject.repo_sync.sync_faq}
                                <span class="variant-filled-primary badge">FAQ sync</span>
                            {/if}
                            {#if $currentProject.repo_sync.sync_links}
                                <span class="variant-filled-primary badge">Links sync</span>
                            {/if}
                            {#if $currentProject.repo_sync.sync_releases}
                                <span class="variant-filled-primary badge">Release uploads</span>
                            {/if}
                        </div>

                        {#if $currentProject.repo_sync.last_push_sync_at}
                            <p>
                                Last content sync:
                                {formatDate(new Date($currentProject.repo_sync.last_push_sync_at))}
                            </p>
                        {/if}

                        {#if $currentProject.repo_sync.last_release_sync_at}
                            <p>
                                Last release sync:
                                {formatDate(new Date($currentProject.repo_sync.last_release_sync_at))}
                            </p>
                        {/if}

                        {#if $currentProject.repo_sync.last_error}
                            <div class="rounded-xl border border-red-500/30 bg-red-500/10 p-3 text-red-100">
                                {$currentProject.repo_sync.last_error}
                            </div>
                        {/if}
                    </div>
                </section>
            {/if}

            {#if $currentProject?.repo_links}
                <section class="card h-fit p-4">
                    <dt class="mb-2 text-sm opacity-50">Repository Links</dt>
                    <dd class="style-markdown flex flex-col overflow-x-auto *:select-text">
                        {@html markdown($currentProject.repo_links)}
                    </dd>
                </section>
            {/if}

            {#if $currentProject?.faq}
                <section class="card h-fit p-4">
                    <dt class="mb-2 text-sm opacity-50">FAQ</dt>
                    <dd class="style-markdown flex flex-col overflow-x-auto *:select-text">
                        {@html markdown($currentProject.faq)}
                    </dd>
                </section>
            {/if}

            {#if installManifest}
                <section class="card h-fit p-4 xl:col-span-2">
                    <div class="mb-3 flex items-center justify-between gap-3">
                        <div>
                            <dt class="text-sm opacity-50">Install Manifest</dt>
                            <h2 class="text-xl font-bold">Repository installer metadata</h2>
                        </div>

                        <span class="variant-soft-secondary badge">.openmods/install.json</span>
                    </div>

                    <pre class="hide-scrollbar overflow-x-auto rounded-xl border border-white/10 bg-black/20 p-4 text-sm leading-6">{installManifest}</pre>
                </section>
            {/if}
        </div>
    {/if}
</div>
