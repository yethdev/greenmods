import type { ModalStore } from "@skeletonlabs/skeleton";

export type ModalCallback = () => void | Promise<void>;

export interface GenericDeleteModalProps {
    message: string;
    callback: ModalCallback;
    afterClose?: ModalCallback;
}

export const showGenericDeleteModal = (store: ModalStore, props: GenericDeleteModalProps) => {
    store.trigger({
        type: "component",
        component: "confirmDeleteGeneric",
        meta: props,
    });
};
