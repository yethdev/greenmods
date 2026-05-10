import type { FullProject, ProjectVisibility } from "./project";
import type { User } from "./user";

export interface ProjectCollection {
    id: number;
    owner: User;
    slug: string;
    name: string;
    description: string;
    readme: string;
    project_ids: number[];
    projects: FullProject[];
    visibility: ProjectVisibility;

    /**
     * Can be converted to a {@link Date}.
     */
    created_at: string;

    /**
     * Can be converted to a {@link Date}.
     */
    updated_at: string;
}

export interface ProjectCollectionInit {
    slug: string;
    name: string;
    description: string;
    readme: string;
    projects: Array<string | number>;
    visibility?: ProjectVisibility;
}

export interface ProjectCollectionPatch {
    slug?: string;
    name?: string;
    description?: string;
    readme?: string;
    projects?: Array<string | number>;
    visibility?: ProjectVisibility;
}