import {
    ErrorResponse,
    unwrapOrNull,
    type Facet,
    type FullProject,
    type SearchResults,
} from "@modhost/api";
import { get, writable } from "svelte/store";
import type { LoadingState, Vec2 } from "./types";
import { client } from "./api";
import { userPreferencesStore } from "./user";
import type { ProjectVersion, SortDirection, SortMode } from "@modhost/api";

export const emptySearchResults: SearchResults = {
    page: 1,
    pages: 0,
    hits: 0,
    total: 0,
    results: [],
};

export const editSaving = writable<boolean>(false);
export const editLoadingState = writable<LoadingState>("loading");
export const currentProject = writable<FullProject | null>(null);
export const currentScrollPosition = writable<Vec2>({ x: 0, y: 0 });
export const popupsDidMount = writable<boolean>(false);

export const currentPage = writable<number>(1);
export const currentQuery = writable<string>("");
export const currentFilters = writable<Facet[]>([]);
export const searchResults = writable<SearchResults>(emptySearchResults);
export const apiAvailable = writable<boolean>(true);

const unwrapTracked = <T>(response: T | ErrorResponse): T | null => {
    if (response instanceof ErrorResponse) {
        apiAvailable.set(false);
        return null;
    }

    apiAvailable.set(true);
    return response;
};

export const updateSearchResults = async (force = false) => {
    if (get(searchResults).hits == 0 || force)
        searchResults.set(await performSearch());

    return get(searchResults).hits != 0;
};

export const searchProjects = async (
    query: string | undefined,
    page: number,
    perPage: number,
    sort: SortMode,
    dir: SortDirection,
    filters: Facet[],
) => {
    const remote = unwrapTracked(await client.search(query, page, perPage, sort, dir, filters));

    return remote ?? emptySearchResults;
};

export const getProjectForDisplay = async (id: string | number): Promise<FullProject | null> =>
    unwrapTracked(await client.project(id).get()) ?? null;

export const getProjectVersionsForDisplay = async (
    id: string | number,
): Promise<ProjectVersion[]> =>
    unwrapTracked(await client.project(id).versions().list()) ?? [];

export const performSearch = async () => {
    const prefs = get(userPreferencesStore);
    const query = get(currentQuery);

    return await searchProjects(
        query == "" ? undefined : query,
        get(currentPage),
        prefs.perPage,
        prefs.sortBy,
        prefs.sortDir,
        get(currentFilters),
    );
};
