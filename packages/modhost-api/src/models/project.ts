import type { User } from "./user";

export type ProjectVisibility = "Public" | "Private" | "Unlisted";

export interface ProjectRepoSync {
    repo_owner: string;
    repo_name: string;
    default_branch?: string;
    sync_readme: boolean;
    sync_releases: boolean;
    sync_faq: boolean;
    sync_links: boolean;
    last_push_sync_at?: string;
    last_release_sync_at?: string;
    last_error?: string;
}

export interface ProjectRepoSyncAdminData extends ProjectRepoSync {
    webhook_path: string;
    webhook_secret: string;
}

export interface ProjectRepoSyncPatch {
    default_branch?: string;
    sync_readme?: boolean;
    sync_releases?: boolean;
    sync_faq?: boolean;
    sync_links?: boolean;
}

export interface Project {
    id: number;
    slug: string;
    name: string;
    description: string;
    downloads: number;
    faq?: string;
    install_json?: string;
    issues?: string;
    license?: string;
    readme: string;
    repo_links?: string;
    repo_sync?: ProjectRepoSync;
    source?: string;
    tags?: string[];
    visibility: ProjectVisibility;
    wiki?: string;

    /**
     * Can be converted to a {@link Date}.
     */
    created_at: string;

    /**
     * Can be converted to a {@link Date}.
     */
    updated_at: string;
}

export interface FullProject extends Project {
    authors: User[];
}

export interface ProjectInit {
    name: string;
    slug: string;
    readme: string;
    description: string;
    issues?: string;
    license?: string;
    source?: string;
    tags?: string[];
    visibility?: ProjectVisibility;
    wiki?: string;
}
