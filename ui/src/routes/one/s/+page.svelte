<script lang="ts">
    import LibrarySearchPage from "$components/ui/LibrarySearchPage.svelte";
    import { siteConfig } from "$lib/config";
    import type { Facet } from "@modhost/api";

    const defaultFacets: Facet[] = [["tags", ["subnautica-1"]]];
    const allowedLoaderIds = ["bepinex", "manual", "qmod"];
    const allowedVersionIds = ["Legacy", "Living Large (2.0)"];
    const hiddenTagIds = ["subnautica-1", "requires-ue4ss", "pak-only"];
    const pageTitle = `Subnautica 1 Search | ${siteConfig.siteName}`;
    const pageDescription =
        "Search and filter Subnautica 1 mods, compatibility data, tags, and release information.";
</script>

<LibrarySearchPage
    routeId="/one/s"
    routePath="/one/s"
    canonicalPath="/one/s"
    {pageTitle}
    {pageDescription}
    {defaultFacets}
    {allowedLoaderIds}
    {allowedVersionIds}
    {hiddenTagIds}
/>
