<script lang="ts">
    import { goto } from "$app/navigation";
    import { base } from "$app/paths";
    import { page } from "$app/stores";
    import { getToastStore } from "@skeletonlabs/skeleton";
    import { onMount } from "svelte";

    const toastStore = getToastStore();

    onMount(() => {
        toastStore.trigger({
            message: `${$page.error?.message}`,
            hideDismiss: true,
            timeout: 5000,
            background: "variant-filled-error",
        });

        setTimeout(() => {
            goto(base + "/");
        }, 5000);
    });
</script>

<div class="container mx-auto flex min-h-full max-w-screen-md items-center justify-center p-4">
    <div class="card w-full space-y-3 p-6 text-center">
        <p class="text-sm uppercase tracking-[0.18em] opacity-50">Request failed</p>
        <h1 class="text-2xl font-bold">Something went wrong.</h1>
        <p class="opacity-80">You will be sent back to the homepage in a few seconds.</p>
        <a href={base + "/"} class="variant-soft-primary btn mx-auto">Go home now</a>
    </div>
</div>
