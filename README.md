# greenmods

greenmods is a Subnautica 2 mod library forked from ModHost. It keeps the fast Rust API, Svelte UI, Meilisearch-backed discovery, moderation queue, gallery support, API client, and self-hosted deployment model, then tightens the product around Subnautica 2.

## Focus

- Green GM identity and calmer interface defaults.
- Subnautica 2 loaders, versions, tags, and file formats.
- Required compatibility metadata on uploaded versions.
- Required creator tags, including tested status.
- Safer authentication, upload validation, and abuse controls.

## Running

Compile the config, start the backing services, then run the greenmods app:

```sh
./compile_config.sh
cargo run -p greenmods
```

Use `config.pkl.example` as the deployment starting point.

The built-in Subnautica 2 profile currently exposes `early-access-2026.05`, `early-access`, and `preview` as compatibility targets. Creators should select the most specific target they have tested against.
