<script lang="ts">
    import { _ } from "svelte-i18n";
    import { page } from "$app/stores";
    import { onMount } from "svelte";
    import { copyText } from "$lib/util";
    import { currentProject, editSaving } from "$lib/state";
    import Icon from "@iconify/svelte";
    import { Autocomplete, getModalStore, getToastStore, popup } from "@skeletonlabs/skeleton";
    import { unwrapOrNull } from "@modhost/api";
    import type { AutocompleteOption, PopupSettings } from "@skeletonlabs/skeleton";
    import type { ProjectRepoSyncAdminData, ProjectVisibility } from "@modhost/api";
    import { licenses } from "$lib/meta";
    import { client } from "$lib/api";

    const id = $derived($page.params.id);
    const modals = getModalStore();
    const toasts = getToastStore();

    let slug = $state("");
    let name = $state("");
    let repo = $state("");
    let issues = $state("");
    let wiki = $state("");
    let license = $state("");
    let visibility = $state<ProjectVisibility>("Public");
    let allLicenses = $state<AutocompleteOption<string, string>[]>([]);
    let gitHubSync = $state<ProjectRepoSyncAdminData | null>(null);
    let gitHubBranch = $state("");
    let syncReadme = $state(true);
    let syncReleases = $state(true);
    let syncFaq = $state(true);
    let syncLinks = $state(true);

    const realRepo = $derived(repo != "" ? repo : undefined);
    const realIssues = $derived(issues != "" ? issues : undefined);
    const realWiki = $derived(wiki != "" ? wiki : undefined);
    const realLicense = $derived(license != "" ? license : undefined);
    const hasGitHubRepo = $derived(repo.toLowerCase().includes("github.com/"));
    const webhookUrl = $derived(gitHubSync ? `${$page.url.origin}${gitHubSync.webhook_path}` : "");

    const applyGitHubSync = (data: ProjectRepoSyncAdminData | null) => {
        gitHubSync = data;
        gitHubBranch = data?.default_branch ?? "";
        syncReadme = data?.sync_readme ?? true;
        syncReleases = data?.sync_releases ?? true;
        syncFaq = data?.sync_faq ?? true;
        syncLinks = data?.sync_links ?? true;
    };

    const loadGitHubSync = async () => {
        if (!repo.toLowerCase().includes("github.com/")) {
            applyGitHubSync(null);
            return;
        }

        applyGitHubSync(unwrapOrNull(await client.project(id).github().get()));
    };

    onMount(async () => {
        if (!$currentProject) return;

        slug = $currentProject.slug;
        name = $currentProject.name;
        repo = $currentProject.source ?? "";
        issues = $currentProject.issues ?? "";
        wiki = $currentProject.wiki ?? "";
        license = $currentProject.license ?? "";
        visibility = $currentProject.visibility;

        allLicenses = $licenses.map((v) => ({ value: v, label: v }));
        await loadGitHubSync();
    });

    const save = async () => {
        $editSaving = true;

        await client.project(id).update({
            name,
            visibility,
            source: realRepo,
            issues: realIssues,
            wiki: realWiki,
            license: realLicense,
        });

        $currentProject = unwrapOrNull(await client.project(id).get());

        slug = $currentProject?.slug ?? slug;
        name = $currentProject?.name ?? name;
        repo = $currentProject?.source ?? repo;
        issues = $currentProject?.issues ?? issues;
        wiki = $currentProject?.wiki ?? wiki;
        license = $currentProject?.license ?? license;
        visibility = $currentProject?.visibility ?? visibility;

        await loadGitHubSync();

        $editSaving = false;
    };

    const saveGitHubSync = async () => {
        if (!gitHubSync) return;

        const data = unwrapOrNull(
            await client.project(id).github().update({
                default_branch: gitHubBranch || undefined,
                sync_readme: syncReadme,
                sync_releases: syncReleases,
                sync_faq: syncFaq,
                sync_links: syncLinks,
            }),
        );

        applyGitHubSync(data);
    };

    const deleteProject = async () => {
        modals.trigger({
            type: "component",
            component: "confirmDelete",
        });
    };

    const licensesPopup: PopupSettings = {
        event: "focus-click",
        target: "licensesAutocomplete",
        placement: "bottom",
    };

    const onLicenseSelect = (ev: CustomEvent<AutocompleteOption<string, string>>) => {
        license = ev.detail.value;
    };
</script>

<p class="text-primary-500 mb-2 flex flex-row items-center justify-start">
    <Icon icon="tabler:settings" height="24" class="mr-2" />
    General Settings
</p>

<div class="card variant-glass-surface w-full p-4">
    <p class="text-primary-500 mb-2 flex flex-row items-center justify-start">
        <Icon icon="tabler:link" height="24" class="mr-2" />
        Slug
    </p>

    <input
        type="text"
        placeholder="Example: my-package"
        class="input rounded-md"
        bind:value={slug}
        disabled
    />
</div>

<div class="card variant-glass-surface w-full p-4">
    <p class="text-primary-500 mb-2 flex flex-row items-center justify-start">
        <Icon icon="tabler:eye" height="24" class="mr-2" />
        Display Name
    </p>

    <input
        type="text"
        placeholder="Example: My Package"
        class="input rounded-md"
        bind:value={name}
    />
</div>

<div class="card variant-glass-surface w-full p-4">
    <p class="text-primary-500 mb-2 flex flex-row items-center justify-start">
        <Icon icon="tabler:code" height="24" class="mr-2" />
        Source Code
    </p>

    <input
        type="text"
        placeholder="Example: https://github.com/example/example"
        class="input rounded-md"
        bind:value={repo}
    />
</div>

{#if hasGitHubRepo}
    <div class="card variant-glass-surface w-full p-4">
        <div class="mb-4 flex items-start justify-between gap-3">
            <div>
                <p class="text-primary-500 mb-2 flex flex-row items-center justify-start">
                    <Icon icon="tabler:brand-github" height="24" class="mr-2" />
                    GitHub Sync
                </p>
                <p class="text-sm opacity-70">
                    Save the project after changing the source URL to generate webhook credentials.
                </p>
            </div>

            <span class="variant-filled-secondary badge">GitHub</span>
        </div>

        {#if gitHubSync}
            <div class="grid gap-4 xl:grid-cols-2">
                <div class="space-y-4">
                    <div>
                        <label class="mb-2 block text-sm opacity-60" for="github-webhook-url">Webhook URL</label>
                        <div class="flex gap-2">
                            <input id="github-webhook-url" class="input rounded-md" readonly value={webhookUrl} />
                            <button
                                type="button"
                                class="variant-soft-secondary btn border border-white/10"
                                onclick={() => copyText(webhookUrl, toasts)}
                            >
                                Copy
                            </button>
                        </div>
                    </div>

                    <div>
                        <label class="mb-2 block text-sm opacity-60" for="github-webhook-secret">Webhook Secret</label>
                        <div class="flex gap-2">
                            <input id="github-webhook-secret" class="input rounded-md" readonly value={gitHubSync.webhook_secret} />
                            <button
                                type="button"
                                class="variant-soft-secondary btn border border-white/10"
                                onclick={() => copyText(gitHubSync?.webhook_secret ?? "", toasts)}
                            >
                                Copy
                            </button>
                        </div>
                    </div>

                    <div>
                        <label class="mb-2 block text-sm opacity-60" for="github-default-branch">Tracked Default Branch</label>
                        <input id="github-default-branch" class="input rounded-md" bind:value={gitHubBranch} placeholder="main" />
                    </div>
                </div>

                <div class="space-y-3 rounded-xl border border-white/10 p-4">
                    <label class="flex items-center justify-between gap-3">
                        <span>Sync README on push</span>
                        <input type="checkbox" class="checkbox" bind:checked={syncReadme} />
                    </label>

                    <label class="flex items-center justify-between gap-3">
                        <span>Sync FAQ on push</span>
                        <input type="checkbox" class="checkbox" bind:checked={syncFaq} />
                    </label>

                    <label class="flex items-center justify-between gap-3">
                        <span>Sync repo links on push</span>
                        <input type="checkbox" class="checkbox" bind:checked={syncLinks} />
                    </label>

                    <label class="flex items-center justify-between gap-3">
                        <span>Upload release assets automatically</span>
                        <input type="checkbox" class="checkbox" bind:checked={syncReleases} />
                    </label>

                    <button
                        type="button"
                        class="variant-filled-primary btn mt-2"
                        onclick={saveGitHubSync}
                    >
                        <Icon icon="tabler:device-floppy" height="20" class="mr-2" />
                        Save GitHub Sync
                    </button>

                    {#if gitHubSync.last_error}
                        <div class="rounded-xl border border-red-500/30 bg-red-500/10 p-3 text-sm text-red-100">
                            {gitHubSync.last_error}
                        </div>
                    {/if}
                </div>
            </div>
        {:else}
            <div class="rounded-xl border border-white/10 bg-black/10 p-4 text-sm opacity-80">
                This project points to GitHub, but sync credentials do not exist yet. Save the project
                once and reload this page to copy the webhook URL and secret.
            </div>
        {/if}
    </div>
{/if}

<div class="card variant-glass-surface w-full p-4">
    <p class="text-primary-500 mb-2 flex flex-row items-center justify-start">
        <Icon icon="tabler:exclamation-circle" height="24" class="mr-2" />
        Issue Tracker
    </p>

    <input
        type="text"
        placeholder="Example: https://github.com/example/example/issues"
        class="input rounded-md"
        bind:value={issues}
    />
</div>

<div class="card variant-glass-surface w-full p-4">
    <p class="text-primary-500 mb-2 flex flex-row items-center justify-start">
        <Icon icon="tabler:world" height="24" class="mr-2" />
        Wiki
    </p>

    <input
        type="text"
        placeholder="Example: https://github.com/example/example/wiki"
        class="input rounded-md"
        bind:value={wiki}
    />
</div>

<div class="card variant-glass-surface w-full p-4">
    <p class="text-primary-500 mb-2 flex flex-row items-center justify-start">
        <Icon icon="tabler:license" height="24" class="mr-2" />
        License
    </p>

    <input
        type="text"
        name="autocomplete-license"
        placeholder="Choose a license (or type your own)"
        class="autocomplete input rounded-md"
        bind:value={license}
        use:popup={licensesPopup}
    />

    <div
        data-popup="licensesAutocomplete"
        class="bg-secondary-700 h-[50%] w-[40%] overflow-scroll rounded-lg p-2"
    >
        <Autocomplete bind:input={license} options={allLicenses} on:selection={onLicenseSelect} />
    </div>
</div>

<div class="card variant-glass-surface w-full p-4">
    <p class="text-primary-500 mb-2 flex flex-row items-center justify-start">
        <Icon icon="tabler:eye" height="24" class="mr-2" />
        Visibility
    </p>

    <select class="select cursor-pointer rounded-lg !outline-none" bind:value={visibility}>
        <option value="Public">Public</option>
        <option value="Private">Private</option>
        <option value="Unlisted">Unlisted</option>
    </select>
</div>

<div class="flex flex-row items-center justify-start gap-2">
    <button
        type="button"
        class="variant-filled-primary btn hover:variant-ghost-primary hover:text-token mt-2 flex flex-row items-center justify-center rounded-lg transition-all"
        onclick={save}
    >
        <Icon icon="tabler:device-floppy" height="24" class="mr-2" />
        Save
    </button>

    <button
        type="button"
        class="variant-filled-error btn hover:variant-ghost-error mt-2 flex flex-row items-center justify-center rounded-lg transition-all"
        onclick={deleteProject}
    >
        <Icon icon="tabler:trash" height="24" class="mr-2" />
        Delete Project
    </button>
</div>
