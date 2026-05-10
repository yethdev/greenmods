<script lang="ts">
    import BetterStep from "$components/ui/stepper/BetterStep.svelte";
    import Icon from "@iconify/svelte";

    interface Props {
        repo: string;
        issues: string;
        wiki: string;
    }

    let { repo = $bindable(), issues = $bindable(), wiki = $bindable() }: Props = $props();
</script>

{#snippet header()}
    <p class="text-primary-500 mb-2 flex flex-row items-center justify-start">
        <Icon icon="tabler:link" height="24" class="mr-2" />
        Links
    </p>
{/snippet}

<BetterStep {header}>
    <div class="card variant-soft-primary w-full p-4">
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

        <div class="mt-4 rounded-xl border border-white/10 bg-black/10 p-4 text-sm leading-6 opacity-80">
            Link a public GitHub repository to unlock repo-native publishing. greenmods can sync
            your README, FAQ, links, and release assets from files like <strong>README.md</strong>,
            <strong>.openmods/FAQ.md</strong>, <strong>.openmods/LINKS.md</strong>, and
            <strong>.openmods/install.json</strong>.
        </div>
    </div>

    <div class="card variant-soft-primary w-full p-4">
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

    <div class="card variant-soft-primary w-full p-4">
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
</BetterStep>
