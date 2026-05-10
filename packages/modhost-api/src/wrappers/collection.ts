import type { Client } from "../client";
import type { ProjectCollectionPatch } from "../models";

export class CollectionWrapper {
    private _client: Client;
    private _collection: string | number;

    public constructor(client: Client, collection: string | number) {
        this._client = client;
        this._collection = collection;
    }

    public get() {
        return this._client.getCollection(this._collection);
    }

    public update(data: ProjectCollectionPatch) {
        return this._client.updateCollection(this._collection, data);
    }

    public delete() {
        return this._client.deleteCollection(this._collection);
    }
}