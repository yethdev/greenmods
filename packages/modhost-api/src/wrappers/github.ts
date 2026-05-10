import type { Client } from "../client";
import type { ProjectRepoSyncPatch } from "../models";

export class ProjectGitHubWrapper {
    private _client: Client;
    private _project: string | number;

    public constructor(client: Client, project: string | number) {
        this._client = client;
        this._project = project;
    }

    public get() {
        return this._client.getProjectGitHubSync(this._project);
    }

    public update(data: ProjectRepoSyncPatch) {
        return this._client.updateProjectGitHubSync(this._project, data);
    }
}