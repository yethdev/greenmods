import type { Client } from "../client";

export class AdminProjectWrapper {
    private _client: Client;
    private _project: string | number;

    public constructor(client: Client, project: string | number) {
        this._client = client;
        this._project = project;
    }

    public delete() {
        return this._client.adminDeleteProject(this._project);
    }
}
