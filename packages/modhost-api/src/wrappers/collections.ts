import type { Client } from "../client";
import type { ProjectCollectionInit } from "../models";

export class CollectionsWrapper {
    private _client: Client;

    public constructor(client: Client) {
        this._client = client;
    }

    public list() {
        return this._client.listCollections();
    }

    public create(data: ProjectCollectionInit) {
        return this._client.createCollection(data);
    }
}