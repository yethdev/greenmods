import type { Project, User } from "@modhost/api";

type ProjectWithOptionalAuthors = Project & {
    authors?: User[];
};

export interface ProjectCreator {
    kind: "nexus" | "site";
    name: string;
    githubId?: number;
    href?: string;
    external?: boolean;
}

const nexusSourcePattern = /^https?:\/\/(?:www\.)?nexusmods\.com\//i;
const importedAuthorPattern = /(?:^|\r?\n)(?:-\s*)?Original author:\s*(.+?)\s*(?=\r?\n|$)/i;

export const isNexusSource = (value?: string | null) => !!value && nexusSourcePattern.test(value);

export const getImportedProjectAuthor = (project: Pick<Project, "readme" | "source">) => {
    if (!isNexusSource(project.source)) return null;

    const match = project.readme.match(importedAuthorPattern);

    return match?.[1]?.trim() || null;
};

export const getProjectCreators = (project: ProjectWithOptionalAuthors): ProjectCreator[] => {
    const importedAuthor = getImportedProjectAuthor(project);

    if (importedAuthor) {
        return [
            {
                kind: "nexus",
                name: importedAuthor,
                href: project.source,
                external: true,
            },
        ];
    }

    return (project.authors ?? []).map((author) => ({
        kind: "site",
        name: author.username,
        githubId: author.github_id,
        href: `/u/${author.username}`,
    }));
};

export const getPrimaryProjectCreator = (project: ProjectWithOptionalAuthors) => {
    return getProjectCreators(project)[0] ?? null;
};