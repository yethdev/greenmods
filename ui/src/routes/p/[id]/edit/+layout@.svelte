<script lang="ts">
    import EditContainer from "$components/ui/edit/EditContainer.svelte";
    import { onMount, tick } from "svelte";
    import type { Snippet } from "svelte";
    import { page } from "$app/stores";
    import { currentProject, editLoadingState } from "$lib/state";
    import { beforeNavigate, goto } from "$app/navigation";
    import { editRoutes } from "$lib/routes";
    import { client } from "$lib/api";
    import { unwrapOrNull } from "@modhost/api";
    import { user } from "$lib/user";

    const id = $derived($page.params.id);
    const ok = $derived(!!$currentProject?.authors.find((v) => v.id == $user?.id) || $user?.admin);

    onMount(async () => {
        $currentProject = unwrapOrNull(await client.project(id).get());

        if ($currentProject) {
            $editLoadingState = "ready";
        } else {
            $editLoadingState = "failed";
        }

        await tick();

        if (!ok) {
            goto(`/p/${id}`);
        }
    });

    beforeNavigate(({ to }) => {
        if (!editRoutes.includes(to?.route.id ?? "")) {
            $currentProject = null;
            $editLoadingState = "loading";
        }
    });

    const { children }: { children: Snippet } = $props();
</script>

{#if ok}
    <EditContainer>
        {#key $page.url.href}
            {@render children?.()}
        {/key}
    </EditContainer>
{/if}
