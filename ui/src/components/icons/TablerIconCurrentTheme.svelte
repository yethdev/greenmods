<script lang="ts">
    import { userPreferencesStore } from "$lib/user";
    import Icon from "@iconify/svelte";
</script>

{#if $userPreferencesStore.lightMode}
    <Icon icon="tabler:sun" height="24" />
{:else}
    <Icon icon="tabler:moon" height="24" />
{/if}
