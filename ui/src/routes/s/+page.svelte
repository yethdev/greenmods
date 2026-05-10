<script lang="ts">
    import LibrarySearchPage from "$components/ui/LibrarySearchPage.svelte";
    import { siteConfig } from "$lib/config";
    import type { Facet } from "@modhost/api";

    const defaultFacets: Facet[] = [["exclude_tags", ["subnautica-1", "subnautica-below-zero"]]];
    const allowedLoaderIds = ["ue4ss", "paks", "manual"];
    const allowedVersionIds = ["early-access-2026.05", "early-access", "preview"];
    const hiddenTagIds = ["subnautica-1", "subnautica-below-zero"];
    const pageTitle = `${siteConfig.siteName} Search | Subnautica 2 mod library`;
    const pageDescription =
        "Search Subnautica 2 mods by loader, preview branch, tags, and release files in the main greenmods library.";
</script>

<LibrarySearchPage
    routeId="/s"
    routePath="/s"
    canonicalPath="/s"
    {pageTitle}
    {pageDescription}
    {defaultFacets}
    {allowedLoaderIds}
    {allowedVersionIds}
    {hiddenTagIds}
/>
