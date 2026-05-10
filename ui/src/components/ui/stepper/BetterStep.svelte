<!-- Reference: https://dribbble.com/shots/16221169-Figma-Material-Ui-components-Steppers-and-sliders -->

<script lang="ts">
    import { getContext, onDestroy } from "svelte";
    import type { Writable } from "svelte/store";
    import type { Snippet } from "svelte";
    import { dynamicTransition } from "$lib/skeleton/transitions";

    // Types
    import type { StepperState } from "./types";
    import type { CssClasses, Transition, TransitionParams } from "@skeletonlabs/skeleton";
    import type { HTMLAttributes } from "svelte/elements";
    type TransitionIn = $$Generic<Transition>;
    type TransitionOut = $$Generic<Transition>;

    interface Props {
        locked?: boolean;
        /** Provide arbitrary classes to the step header region. */
        regionHeader?: CssClasses;
        /** Provide arbitrary classes to the step content region. */
        regionContent?: CssClasses;
        /** Provide arbitrary classes to the step navigation region. */
        regionNavigation?: CssClasses;
        state?: Writable<StepperState>;
        stepTerm?: string;
        gap?: CssClasses;
        justify?: CssClasses;
        onNext?: (locked: boolean, stepIndex: number) => void;
        onBack?: (stepIndex: number) => void;
        onComplete?: (stepIndex: number) => void;
        buttonBack?: Snippet<[boolean, () => void]>;
        buttonNext?: Snippet<[boolean, () => void]>;
        buttonComplete?: Snippet<[boolean, () => void]>;
        /** Enable/Disable transitions */
        transitions?: boolean;
        /** Provide the transition to used on entry. */
        transitionIn?: TransitionIn;
        /** Transition params provided to `transitionIn`. */
        transitionInParams?: TransitionParams<TransitionIn>;
        /** Provide the transition to used on exit. */
        transitionOut?: TransitionOut;
        /** Transition params provided to `transitionOut`. */
        transitionOutParams?: TransitionParams<TransitionOut>;
        header?: Snippet;
        navigation?: Snippet;
    }

    let {
        locked = false,
        regionHeader = "",
        regionContent = "",
        regionNavigation = "",
        state = getContext("state")!,
        stepTerm = getContext("stepTerm")!,
        gap = getContext("gap")!,
        justify = getContext("justify")!,
        onNext = getContext("onNext")!,
        onBack = getContext("onBack")!,
        onComplete = getContext("onComplete")!,
        buttonBack = getContext("buttonBack")!,
        buttonNext = getContext("buttonNext")!,
        buttonComplete = getContext("buttonComplete")!,
        transitions = getContext("transitions")!,
        transitionIn = getContext("transitionIn")!,
        transitionInParams = getContext("transitionInParams")!,
        transitionOut = getContext("transitionOut")!,
        transitionOutParams = getContext("transitionOutParams")!,
        header,
        children,
        navigation,
        ...restProps
    }: Props & HTMLAttributes<HTMLElement> = $props();

    // Register step on init (keep these paired)
    const stepIndex = $state.total;

    $state.total++;

    // Classes
    const cBase = "space-y-4";
    const cHeader = "text-2xl font-bold";
    const cContent = "space-y-4";
    const cNavigation = "flex";

    // Reactive
    const classesBase = $derived(`${cBase} ${restProps.class ?? ""}`);
    const classesHeader = $derived(`${cHeader} ${regionHeader}`);
    const classesContent = $derived(`${cContent} ${regionContent}`);
    const classesNavigation = $derived(`${cNavigation} ${justify} ${gap} ${regionNavigation}`);

    // Unregister step on destroy
    onDestroy(() => {
        $state.total--;
    });
</script>

{#if stepIndex === $state.current}
    <div class="step {classesBase}" data-testid="step">
        <!-- Slot: Header -->
        <header class="step-header {classesHeader}">
            {#if header}
                {@render header()}
            {:else}
                {stepTerm} {stepIndex + 1}
            {/if}
        </header>

        <!-- Slot: Default -->
        <div class="step-content {classesContent}">
            {#if children}
                {@render children()}
            {:else}
                ({stepTerm} {stepIndex + 1} Content)
            {/if}
        </div>

        <!-- Navigation -->
        {#if $state.total > 1}
            <div
                class="step-navigation {classesNavigation}"
                in:dynamicTransition|local={{
                    transition: transitionIn,
                    params: transitionInParams,
                    enabled: transitions,
                }}
                out:dynamicTransition|local={{
                    transition: transitionOut,
                    params: transitionOutParams,
                    enabled: transitions,
                }}
            >
                {#if stepIndex === 0 && navigation}
                    <!-- Slot: Navigation -->
                    <div class="step-navigation-slot">
                        {@render navigation()}
                    </div>
                {:else}
                    {@render buttonBack($state.current === 0, () => onBack(stepIndex))}
                {/if}

                {#if stepIndex < $state.total - 1}
                    {@render buttonNext(locked, () => onNext(locked, stepIndex))}
                {:else}
                    {@render buttonComplete(locked, () => onComplete(stepIndex))}
                {/if}
            </div>
        {/if}
    </div>
{/if}
