<script lang="ts">
    import LibrarySearchPage from "$components/ui/LibrarySearchPage.svelte";
    import { siteConfig } from "$lib/config";
    import type { Facet } from "@modhost/api";

    const defaultFacets: Facet[] = [["tags", ["subnautica-below-zero"]]];
    const allowedLoaderIds = ["bepinex", "manual", "qmod"];
    const allowedVersionIds = ["Below Zero"];
    const hiddenTagIds = ["subnautica-1", "subnautica-below-zero", "requires-ue4ss", "pak-only"];
    const pageTitle = `Below Zero Search | ${siteConfig.siteName}`;
    const pageDescription =
        "Search and filter Below Zero mods, compatibility data, tags, and release information.";
</script>

<LibrarySearchPage
    routeId="/zero/s"
    routePath="/zero/s"
    canonicalPath="/zero/s"
    {pageTitle}
    {pageDescription}
    {defaultFacets}
    {allowedLoaderIds}
    {allowedVersionIds}
    {hiddenTagIds}
/>