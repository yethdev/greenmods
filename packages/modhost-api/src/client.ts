import {
    type ProjectCollection,
    type ProjectCollectionInit,
    type ProjectCollectionPatch,
    ErrorResponse,
    serializeFacets,
    type Facet,
    type FullProject,
    type GalleryImage,
    type GalleryImageInit,
    type GameVersion,
    type ModLoader,
    type ProjectInit,
    type ProjectRepoSyncAdminData,
    type ProjectRepoSyncPatch,
    type ProjectVersion,
    type ProjectVersionInit,
    type SearchResults,
    type SortDirection,
    type SortMode,
    type Tag,
    type User,
} from "./models";
import type { AdminStats } from "./models/admin";

const GET_REQUEST_TIMEOUT_MS = 12_000;
const RETRYABLE_STATUS_CODES = new Set([408, 429, 502, 503, 504]);
const RETRY_BACKOFF_MS = 250;

const delay = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

/**
 * The base API client.
 * This is used internally for requests and isn't as ergonomic as the wrapped client.
 */
export class Client {
    private _baseUrl: string;
    private _token?: string;

    public constructor(baseUrl = "/api/v1", token?: string) {
        this._baseUrl = baseUrl.endsWith("/") ? baseUrl.substring(0, baseUrl.length - 1) : baseUrl;
        this._token = token;
    }

    public setToken(token: string) {
        this._token = token;
    }

    public unsetToken() {
        this._token = undefined;
    }

    public hasToken() {
        return !!this._token;
    }

    public getToken() {
        return this._token;
    }

    private async _fetch(
        requiresAuth: boolean = false,
        method: "GET" | "POST" | "PUT" | "DELETE" | "PATCH",
        path: string,
        headers: Record<string, any> = {},
        body?: Blob | FormData | object | string,
    ) {
        const realPath = path.startsWith("/") ? path.substring(1) : path;
        const isGet = method == "GET";
        const attempts = isGet ? 2 : 1;

        const data =
            !(body instanceof FormData) && typeof body === "object" ? JSON.stringify(body) : body;

        const fullHeaders = {
            ...headers,
        };

        if (this._token) {
            fullHeaders["Authorization"] = `Bearer ${this._token}`;
        }

        if (requiresAuth && !this._token) {
            throw new Error("This request requires an auth token.");
        }

        if (!(body instanceof FormData) && !(body instanceof Blob) && typeof body === "object") {
            fullHeaders["Content-Type"] = "application/json";
        }

        let lastError: unknown = undefined;

        for (let attempt = 0; attempt < attempts; attempt++) {
            const controller = new AbortController();
            const timeout = isGet
                ? setTimeout(() => controller.abort(), GET_REQUEST_TIMEOUT_MS)
                : undefined;

            try {
                const response = await fetch(`${this._baseUrl}/${realPath}`, {
                    method: method,
                    body: data,
                    credentials: "same-origin",
                    headers: fullHeaders,
                    signal: controller.signal,
                });

                if (
                    !isGet ||
                    !RETRYABLE_STATUS_CODES.has(response.status) ||
                    attempt == attempts - 1
                ) {
                    return response;
                }

                lastError = new Error(`Retryable response status: ${response.status}`);
            } catch (err: unknown) {
                lastError = err;

                if (!isGet || attempt == attempts - 1) {
                    throw err;
                }
            } finally {
                if (timeout != undefined) clearTimeout(timeout);
            }

            await delay(RETRY_BACKOFF_MS * (attempt + 1));
        }

        throw (lastError ?? new Error("Request failed."));
    }

    private async _jsonFetch<T>(
        requiresAuth: boolean = false,
        method: "GET" | "POST" | "PUT" | "DELETE" | "PATCH",
        path: string,
        headers: Record<string, any> = {},
        body?: Blob | FormData | object | string,
    ): Promise<T | ErrorResponse> {
        try {
            const res = await this._fetch(requiresAuth, method, path, headers, body);
            const text = await res.text();

            try {
                return JSON.parse(text) as T;
            } catch (err: unknown) {
                return new ErrorResponse(text);
            }
        } catch (err: unknown) {
            return new ErrorResponse(err);
        }
    }

    // =================== USERS ===================

    public async currentUser() {
        return await this._jsonFetch<User>(true, "GET", "/users/me");
    }

    public async getUser(id: number | string) {
        return await this._jsonFetch<User>(false, "GET", `/users/${id}`);
    }

    public async getUserProjects(id: number | string) {
        return await this._jsonFetch<FullProject[]>(false, "GET", `/users/${id}/projects`);
    }

    // =================== COLLECTIONS ===================

    public async listCollections() {
        return await this._jsonFetch<ProjectCollection[]>(false, "GET", "/collections");
    }

    public async getCollection(id: string | number) {
        return await this._jsonFetch<ProjectCollection>(false, "GET", `/collections/${id}`);
    }

    public async createCollection(data: ProjectCollectionInit) {
        return await this._jsonFetch<ProjectCollection>(true, "PUT", "/collections", {}, {
            ...data,
            projects: data.projects.map((project) => project.toString()),
        });
    }

    public async updateCollection(id: string | number, data: ProjectCollectionPatch) {
        return await this._jsonFetch<ProjectCollection>(true, "PATCH", `/collections/${id}`, {}, {
            ...data,
            projects: data.projects?.map((project) => project.toString()),
        });
    }

    public async deleteCollection(id: string | number) {
        return await this._fetch(true, "DELETE", `/collections/${id}`);
    }

    // =================== PROJECTS ===================

    public async searchProjects(
        query?: string,
        page = 1,
        perPage = 25,
        sort: SortMode = "none",
        dir: SortDirection = "desc",
        filters: Facet[] = [],
    ) {
        const queryObj: Record<string, string> = {};

        if (query) queryObj["q"] = encodeURIComponent(query);
        if (sort) queryObj["sort"] = sort;
        if (dir) queryObj["dir"] = dir;

        queryObj["page"] = page.toString();
        queryObj["per_page"] = perPage.toString();
        queryObj["filters"] = serializeFacets(filters);

        let queryStr = "";

        for (const [k, v] of Object.entries(queryObj)) {
            if (queryStr.startsWith("?")) {
                queryStr += `&${k}=${v}`;
            } else {
                queryStr += `?${k}=${v}`;
            }
        }

        return await this._jsonFetch<SearchResults>(false, "GET", `/projects/search${queryStr}`);
    }

    public async getProject(id: string | number) {
        return await this._jsonFetch<FullProject>(false, "GET", `/projects/${id}`);
    }

    public async getProjectAuthors(id: string | number) {
        return await this._jsonFetch<User[]>(false, "GET", `/projects/${id}/authors`);
    }

    public async createProject(data: ProjectInit) {
        return await this._jsonFetch<FullProject>(true, "PUT", "/projects", {}, data);
    }

    public async deleteProject(id: string | number) {
        return await this._fetch(true, "DELETE", `/projects/${id}`);
    }

    public async updateProject(id: string | number, data: Partial<Omit<ProjectInit, "slug">>) {
        return await this._jsonFetch<FullProject>(true, "PATCH", `/projects/${id}`, {}, data);
    }

    public async addProjectAuthor(id: string | number, author: string | number) {
        return await this._jsonFetch<FullProject>(
            true,
            "PUT",
            `/projects/${id}/authors`,
            {},
            author.toString(),
        );
    }

    public async removeProjectAuthor(id: string | number, author: string | number) {
        return await this._jsonFetch<FullProject>(
            true,
            "DELETE",
            `/projects/${id}/authors`,
            {},
            author.toString(),
        );
    }

    public async getProjectGitHubSync(id: string | number) {
        return await this._jsonFetch<ProjectRepoSyncAdminData>(
            true,
            "GET",
            `/projects/${id}/github`,
        );
    }

    public async updateProjectGitHubSync(id: string | number, data: ProjectRepoSyncPatch) {
        return await this._jsonFetch<ProjectRepoSyncAdminData>(
            true,
            "PATCH",
            `/projects/${id}/github`,
            {},
            data,
        );
    }

    // =================== GALLERY ===================

    public async getGalleryImages(project: string | number) {
        return await this._jsonFetch<GalleryImage[]>(false, "GET", `/projects/${project}/gallery`);
    }

    public async getGalleryImage(project: string | number, image: string | number) {
        return await this._jsonFetch<GalleryImage>(
            false,
            "GET",
            `/projects/${project}/gallery/${image}`,
        );
    }

    public async uploadGalleryImage(project: string | number, data: GalleryImageInit) {
        const form = new FormData();

        form.set("name", data.name);
        form.set("ordering", data.ordering.toString());
        form.set("project", data.project.toString());
        form.set("file", data.file);

        if (data.description) form.set("description", data.description);

        return await this._jsonFetch<GalleryImage>(
            true,
            "PUT",
            `/projects/${project}/gallery`,
            {},
            form,
        );
    }

    public async updateGalleryImage(
        project: string | number,
        image: string | number,
        data: Partial<Omit<Omit<GalleryImageInit, "project">, "file">>,
    ) {
        return await this._jsonFetch<GalleryImage>(
            true,
            "PATCH",
            `/projects/${project}/gallery/${image}`,
            {},
            data,
        );
    }

    public async deleteGalleryImage(project: string | number, image: string | number) {
        return await this._fetch(true, "DELETE", `/projects/${project}/gallery/${image}`);
    }

    // =================== VERSIONS ===================

    public async getProjectVersions(project: string | number) {
        return await this._jsonFetch<ProjectVersion[]>(
            false,
            "GET",
            `/projects/${project}/versions`,
        );
    }

    public async getProjectVersion(project: string | number, version: string | number) {
        return await this._jsonFetch<ProjectVersion>(
            false,
            "GET",
            `/projects/${project}/versions/${version}`,
        );
    }

    public async getLatestVersion(project: string | number) {
        return await this._jsonFetch<ProjectVersion>(
            false,
            "GET",
            `/projects/${project}/versions/latest`,
        );
    }

    public async downloadProjectVersion(
        project: string | number,
        version: string | number,
        file: string | number,
    ) {
        return await (
            await this._fetch(
                false,
                "GET",
                `/projects/${project}/versions/${version}/download/${file}`,
            )
        ).arrayBuffer();
    }

    public downloadProjectVersionUrl(
        project: string | number,
        version: string | number,
        file: string | number,
    ) {
        return `${this._baseUrl}/projects/${project}/versions/${version}/download/${file}`;
    }

    public async downloadProjectVersionModOnly(
        project: string | number,
        version: string | number,
        file: string | number,
    ) {
        return await (
            await this._fetch(
                false,
                "GET",
                `/projects/${project}/versions/${version}/download/${file}/mod-only`,
            )
        ).arrayBuffer();
    }

    public downloadProjectVersionModOnlyUrl(
        project: string | number,
        version: string | number,
        file: string | number,
    ) {
        return `${this._baseUrl}/projects/${project}/versions/${version}/download/${file}/mod-only`;
    }

    public async uploadProjectVersion(project: string | number, data: ProjectVersionInit) {
        const form = new FormData();

        form.set("name", data.name);
        form.set("version_number", data.version_number.toString());
        form.set("game_versions", data.game_versions.join(","));
        form.set("loaders", data.loaders.join(","));

        if (data.file_name && data.file) {
            form.set("file_name", data.file_name);
            form.set("file", data.file);
        }

        if (data.changelog) form.set("changelog", data.changelog);

        return await this._jsonFetch<ProjectVersion>(
            true,
            "PUT",
            `/projects/${project}/versions`,
            {},
            form,
        );
    }

    public async updateProjectVersion(
        project: string | number,
        version: string | number,
        data: Partial<Omit<Omit<ProjectVersionInit, "file">, "file_name">>,
    ) {
        return await this._jsonFetch<ProjectVersion>(
            true,
            "PATCH",
            `/projects/${project}/versions/${version}`,
            {},
            data,
        );
    }

    public async deleteProjectVersion(project: string | number, version: string | number) {
        return await this._fetch(true, "DELETE", `/projects/${project}/versions/${version}`);
    }

    // =================== METADATA ===================

    public async getGameVersions() {
        return await this._jsonFetch<GameVersion[]>(false, "GET", "/meta/game_versions");
    }

    public async getModLoaders() {
        return await this._jsonFetch<ModLoader[]>(false, "GET", "/meta/loaders");
    }

    public async getTags() {
        return await this._jsonFetch<Tag[]>(false, "GET", "/meta/tags");
    }

    // =================== ADMIN ===================

    public async getAdminStats() {
        return await this._jsonFetch<AdminStats>(true, "GET", "/admin/stats");
    }

    public async adminGetAllUsers() {
        return await this._jsonFetch<User[]>(true, "GET", "/admin/users/list");
    }

    public async adminGetAllProjects() {
        return await this._jsonFetch<FullProject[]>(true, "GET", "/admin/projects/list");
    }

    public async adminGetUser(user: string | number) {
        return await this._jsonFetch<User>(true, "GET", `/admin/users/${user}`);
    }

    public async adminDeleteUser(user: string | number) {
        return await this._jsonFetch<User>(true, "DELETE", `/admin/users/${user}`);
    }

    public async adminDeleteProject(project: string | number) {
        return await this._fetch(true, "DELETE", `/admin/projects/${project}`);
    }

    public async listAdmins() {
        return await this._jsonFetch<User[]>(true, "GET", "/admin/list");
    }

    public async addAdmin(user: string | number) {
        return await this._fetch(true, "PUT", `/admin/add/${user}`);
    }

    public async removeAdmin(user: string | number) {
        return await this._fetch(true, "DELETE", `/admin/remove/${user}`);
    }
}
