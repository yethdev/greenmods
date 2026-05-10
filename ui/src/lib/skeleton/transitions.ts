import type { TransitionConfig } from "svelte/transition";

export function dynamicTransition<T extends Transition>(
    node: Element,
    dynParams: DynamicTransitionParams<T>,
): TransitionConfig {
    const { transition, params, enabled } = dynParams;

    if (enabled) return transition(node, params);

    // it's better to just set the `duration` to 0 to prevent flickering
    if (params && typeof params === "object" && "duration" in params) {
        return transition(node, { duration: 0 });
    }

    // if the transition doesn't provide a `duration` prop, then we'll just return this as a last resort
    return { duration: 0 };
}

type DynamicTransitionParams<T extends Transition> = {
    transition: T;
    params: TransitionParams<T>;
    enabled: boolean;
};

export type Transition = (node: Element, params?: unknown) => TransitionConfig;
export type TransitionParams<T extends Transition> = Parameters<T>[1];
