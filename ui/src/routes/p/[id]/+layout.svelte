<script lang="ts">
    import { afterNavigate, beforeNavigate } from "$app/navigation";
    import { _ } from "svelte-i18n";
    import { page } from "$app/stores";
    import type { LoadingState } from "$lib/types";
    import {
        fixLoaderName,
        getLoaders,
        getGameVersions,
        isNexusSource,
        markdownInline,
        formatDate,
        copyText,
        getProjectCreators,
    } from "$lib/util";
    import { onMount } from "svelte";
    import { getToastStore } from "@skeletonlabs/skeleton";
    import { base } from "$app/paths";
    import { getProjectPreviewImage } from "$lib/api";
    import { currentProject, getProjectDisplayData } from "$lib/state";
    import { tryAggregateVersions } from "$lib/vers";
    import { siteConfig } from "$lib/config";
    import Icon from "@iconify/svelte";
    import { pkgRoutes } from "$lib/routes";
    import ProjectTabs from "$components/ui/ProjectTabs.svelte";
    import type { FullProject, ProjectVersion, ProjectVisibility, Tag } from "@modhost/api";
    import { user } from "$lib/user";
    import { tags as allTags } from "$lib/meta";
    import { absoluteSiteUrl, trimDescription } from "$lib/seo";

    const maxVersions = 10;
    const id = $derived($page.params.id);
    const toasts = getToastStore();

    let loadingState: LoadingState = $state("loading");
    let versions: ProjectVersion[] = $state([]);

    let name = $state("");
    let repo = $state("");
    let issues = $state("");
    let wiki = $state("");
    let tags = $state<Tag[]>([]);
    let license = $state<string | undefined>(undefined);
    let vis = $state<ProjectVisibility>("Public");
    let imageUrl = $state<string | undefined>(undefined);

    const loaders = $derived(getLoaders(versions));
    const gameVersions = $derived(getGameVersions(versions));

    const hasRepo = $derived(repo != "");
    const hasIssues = $derived(issues != "");
    const hasWiki = $derived(wiki != "");
    const sourceIsNexus = $derived(isNexusSource(repo));
    const projectUrl = $derived(absoluteSiteUrl(`/p/${id}`));
    const projectImage = $derived(imageUrl ?? absoluteSiteUrl("/modhost.png"));
    const projectDescription = $derived(
        trimDescription(
            $currentProject?.description,
            `View loaders, supported game versions, tags, and source links for ${name || id}.`,
        ),
    );
    const projectKeywords = $derived(
        [...new Set([...tags.map((tag) => tag.name), ...loaders.map((loader) => fixLoaderName(loader))])].join(", "),
    );
    const projectJsonLd = $derived(
        !$currentProject
            ? ""
            : JSON.stringify({
                  "@context": "https://schema.org",
                  "@type": "CreativeWork",
                  name: $currentProject.name,
                  description: projectDescription,
                  url: projectUrl,
                  image: projectImage,
                  genre: "Subnautica 2 mod",
                  datePublished: $currentProject.created_at,
                  dateModified: $currentProject.updated_at,
                  keywords: tags.map((tag) => tag.name),
                  author: getProjectCreators($currentProject).map((author) => ({
                      "@type": "Person",
                      name: author.name,
                      url: author.external ? author.href : absoluteSiteUrl(author.href ?? "/"),
                  })),
                  sameAs: [repo, issues, wiki].filter(Boolean),
              }),
    );
    const projectCreators = $derived($currentProject ? getProjectCreators($currentProject) : []);

    const canEdit = $derived(
        ($currentProject && $user && !!$currentProject.authors.find((v) => v.id == $user.id)) ||
            ($user && $user.admin),
    );

    const visibility = $derived(
        vis == "Public" ? "Public" : vis == "Private" ? "Private" : "Unlisted",
    );

    const copyId = async () => {
        if (!$currentProject) return;

        await copyText($currentProject.id.toString(), toasts);
    };

    const applyProjectData = async (project: FullProject | null, nextVersions: ProjectVersion[]) => {
        $currentProject = project;
        versions = nextVersions;

        if (!$currentProject) {
            imageUrl = undefined;
            loadingState = "failed";
            return;
        }

        name = $currentProject.name;
        repo = $currentProject.source ?? "";
        issues = $currentProject.issues ?? "";
        wiki = $currentProject.wiki ?? "";
        license = $currentProject.license;
        vis = $currentProject.visibility;
        tags =
            $currentProject.tags
                ?.filter((t) => !!$allTags.find((v) => v.id == t))
                .map((t) => $allTags.find((v) => v.id == t)!) ?? [];
        imageUrl = await getProjectPreviewImage(id);
        loadingState = "ready";
    };

    onMount(async () => {
        const { project, versions } = await getProjectDisplayData(id);
        await applyProjectData(project, versions);
    });

    beforeNavigate(({ to }) => {
        if (pkgRoutes.includes(to?.route.id ?? "")) return;

        $currentProject = null;
        loadingState = "loading";
    });

    afterNavigate(({ to }) => {
        if (pkgRoutes.includes(to?.route.id ?? "") && !$currentProject) {
            reset();
        }
    });

    const reset = async () => {
        const { project, versions } = await getProjectDisplayData(id);
        await applyProjectData(project, versions);
    };

    const aggVersions = $derived(tryAggregateVersions(gameVersions));
    const { children } = $props();
</script>

<svelte:head>
    <title>{$currentProject?.name ?? $_("site.loading")} | {siteConfig.siteName}</title>
    <meta name="description" content={projectDescription} />
    <link rel="canonical" href={projectUrl} />
    <meta property="og:title" content={$currentProject?.name ?? siteConfig.siteName} />
    <meta property="og:description" content={projectDescription} />
    <meta property="og:url" content={projectUrl} />
    <meta property="og:image" content={projectImage} />
    <meta property="og:image:alt" content={`${$currentProject?.name ?? siteConfig.siteName} preview image`} />
    <meta name="twitter:title" content={$currentProject?.name ?? siteConfig.siteName} />
    <meta name="twitter:description" content={projectDescription} />
    <meta name="twitter:image" content={projectImage} />
    {#if projectKeywords}
        <meta name="keywords" content={projectKeywords} />
    {/if}
    {#if projectJsonLd}
        <script type="application/ld+json">{projectJsonLd}</script>
    {/if}
</svelte:head>

{#if loadingState == "loading"}
    <div class="placeholder m-2 mx-auto w-32 animate-pulse"></div>
{:else if loadingState == "ready" && $currentProject}
    <div class="flex w-full flex-col gap-2 md:flex-row">
        <div
            class="card flex w-full flex-col items-start justify-start gap-2 self-baseline p-4 md:w-[30%]"
        >
            {#if imageUrl}
                <img
                    src={imageUrl}
                    alt={`${name} preview image`}
                    class="aspect-square w-[40%] rounded-lg object-cover"
                    decoding="async"
                />
            {/if}

            <div class="flex w-full flex-row items-center justify-between">
                <a href="/p/{id}" class="text-primary-500 text-2xl font-bold">
                    {name}
                </a>

                <div class="flex flex-row items-center justify-end gap-2">
                    {#if canEdit}
                        <a
                            aria-label="Edit"
                            href="/p/{id}/edit"
                            class="hover:variant-filled-primary flex flex-row items-center justify-center rounded-full p-2 transition-all"
                        >
                            <Icon icon="tabler:pencil" height="24" />
                        </a>
                    {/if}
                </div>
            </div>

            <span
                class="variant-filled-secondary badge flex flex-row items-center justify-center px-2"
            >
                {#if vis == "Public"}
                    <Icon icon="tabler:eye" height="22" class="mr-2" />
                {:else}
                    <Icon icon="tabler:eye-off" height="22" class="mr-2" />
                {/if}
                {visibility}
            </span>

            <hr class="w-full" />

            <span
                class="style-markdown w-full select-text hyphens-auto text-wrap break-words *:select-text"
            >
                {@html markdownInline($currentProject.description)}
            </span>

            <span class="text-sm opacity-50">
                <span
                    >{$currentProject.downloads}
                    {$currentProject.downloads == 1
                        ? $_("list.download_singluar")
                        : $_("list.download_plural")}</span
                >
            </span>

            <hr class="w-full" />

            <dt class="text-sm opacity-50">{$_("package.available_for")}</dt>

            {#if loaders.length > 0 || gameVersions.length > 0}
                {#if loaders.length > 0}
                    <dd class="flex flex-wrap gap-1">
                        {#each loaders as loader}
                            <span class="variant-filled-primary badge select-text"
                                >{fixLoaderName(loader)}</span
                            >
                        {/each}
                    </dd>
                {/if}
                {#if gameVersions.length > 0}
                    <dd class="flex flex-wrap gap-1">
                        {#if aggVersions.length > maxVersions}
                            {#each aggVersions.slice(0, maxVersions) as version}
                                <span class="variant-filled-primary badge select-text"
                                    >{version}</span
                                >
                            {/each}
                            <span class="variant-filled-primary badge select-text">...</span>
                        {:else}
                            {#each aggVersions as version}
                                <span class="variant-filled-primary badge select-text"
                                    >{version}</span
                                >
                            {/each}
                        {/if}
                    </dd>
                {/if}
            {:else}
                <dd class="flex flex-wrap gap-1">
                    <span class="variant-filled-primary badge select-text"
                        >{$_("package.unknown")}</span
                    >
                </dd>
            {/if}

            {#if tags.length > 0}
                <hr class="w-full" />

                <dt class="text-sm opacity-50">Tags</dt>

                <dd class="flex flex-wrap gap-1">
                    {#each tags as tag}
                        <span class="variant-filled-primary badge select-text">
                            <Icon icon={tag.icon} width="16" class="mr-2" />
                            {tag.name}
                        </span>
                    {/each}
                </dd>
            {/if}

            <hr class="w-full" />

            {#if license}
                <p class="text-sm opacity-50">License</p>
                <p>{license}</p>

                <hr class="w-full" />
            {/if}

            {#if hasRepo}
                <a
                    href={repo}
                    class="anchor hover:border-b-primary-500 select-text border-b-2 border-b-transparent no-underline transition-all"
                    target="_blank"
                    rel="noreferrer"
                >
                    {$_("package.source")}
                    {#if sourceIsNexus}
                        <Icon icon="tabler:external-link" class="ml-1 inline-block" width="16" />
                    {/if}
                </a>
            {/if}

            {#if hasIssues}
                <a
                    href={issues}
                    class="anchor hover:border-b-primary-500 select-text border-b-2 border-b-transparent no-underline transition-all"
                    target="_blank"
                    rel="noopener noreferrer"
                >
                    {$_("package.issues")}
                </a>
            {/if}

            {#if hasWiki}
                <a
                    href={wiki}
                    class="anchor hover:border-b-primary-500 select-text border-b-2 border-b-transparent no-underline transition-all"
                    target="_blank"
                    rel="noopener noreferrer"
                >
                    {$_("package.wiki")}
                </a>
            {/if}

            {#if !license && !(hasRepo || hasIssues || hasWiki)}
                <hr class="w-full" />
            {/if}

            <p class="text-sm opacity-50">{$_("package.version.published")}</p>
            <p class="mb-1">{formatDate(new Date($currentProject.created_at))}</p>

            <p class="text-sm opacity-50">{$_("package.version.updated")}</p>
            <p>{formatDate(new Date($currentProject.updated_at))}</p>

            <hr class="w-full" />

            <dt class="text-sm opacity-50">{$_("package.created_by")}</dt>

            {#each projectCreators as author}
                <a
                    class="card hover:variant-soft-primary flex w-full flex-row items-center p-2"
                    href={author.href ?? `${base}/u/unknown`}
                    target={author.external ? "_blank" : undefined}
                    rel={author.external ? "noreferrer" : undefined}
                >
                    {#if author.kind == "nexus"}
                        <span
                            class="bg-[#da8d32] text-black rounded-token my-auto mr-4 flex aspect-square h-8 items-center justify-center"
                            aria-label="Nexus Mods"
                        >
                            <Icon icon="simple-icons:nexusmods" class="h-4 w-4" />
                        </span>
                    {:else if author.githubId == -1}
                        <img
                            src="/modhost.png"
                            alt="author's profile avatar"
                            class="rounded-token my-auto mr-4 aspect-square h-8"
                            loading="lazy"
                            decoding="async"
                        />
                    {:else}
                        <img
                            src="https://avatars.githubusercontent.com/u/{author.githubId}"
                            alt="author's profile avatar"
                            class="rounded-token my-auto mr-4 aspect-square h-8"
                            loading="lazy"
                            decoding="async"
                            referrerpolicy="no-referrer"
                        />
                    {/if}
                    {author.name}
                </a>
            {/each}

            <hr class="w-full" />

            <span class="flex flex-row items-center justify-end">
                {$_(`id.${siteConfig.type}`)}&nbsp;
                <button
                    class="anchor flex select-text flex-row items-center justify-start no-underline"
                    onclick={copyId}
                    >{$currentProject.id}
                    <Icon icon="tabler:copy" class="ml-1" />
                </button>
            </span>
        </div>

        <div class="flex w-full flex-col items-start justify-start gap-2 md:w-[70%]">
            <ProjectTabs
                tabs={[
                    {
                        routes: ["/p/[id]"],
                        text: "Description",
                        url: `/p/${id}`,
                    },
                    {
                        routes: ["/p/[id]/gallery"],
                        text: "Gallery",
                        url: `/p/${id}/gallery`,
                    },
                    {
                        routes: ["/p/[id]/versions", "/p/[id]/versions/[ver]"],
                        text: "Versions",
                        url: `/p/${id}/versions`,
                    },
                ]}
            />

            {@render children?.()}
        </div>
    </div>
{:else if loadingState == "failed"}
    <!-- <p>Something went wrong (this package doesn't seem to exist)</p> -->
    {(() => {
        toasts.trigger({
            message: `Mod/Package Broken`,
            hideDismiss: true,
            timeout: 5000,
            background: "variant-filled-error",
        });

        history.back();

        return undefined;
    })() || "Please wait, redirecting..."}
{:else}
    <!-- <div class="flex flex-col items-center justify-center">
        <span>Something went horribly,&nbsp;<i>horribly</i>&nbsp;wrong.</span>

        <span
            >Try <button type="button" class="anchor" onclick={reset}
                >refreshing the page data</button
            >.</span
        >

        <span
            >If the issue persists, please open an issue on our <a
                class="anchor"
                href="https://github.com/RedstoneWizard08/ModHost/issues/new"
                target="_blank">GitHub</a
            >.</span
        >
    </div> -->
{/if}
