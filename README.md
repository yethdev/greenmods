# GreenMods

GreenMods is a Subnautica 2 mod library forked from ModHost. It keeps the fast Rust API, Svelte UI, Meilisearch-backed discovery, moderation queue, gallery support, API client, and self-hosted deployment model, then tightens the product around Subnautica 2.

## Focus

- Green GM identity and calmer interface defaults.
- Subnautica 2 loaders, versions, tags, and file formats.
- Required compatibility metadata on uploaded versions.
- Required creator tags, including tested status.
- Safer authentication, upload validation, and abuse controls.
- Deployment notes for a 10 vCore ARM64 VPS with 16 GB RAM.

## Running

Compile the config, start the backing services, then run the GreenMods app:

```sh
./compile_config.sh
cargo run -p greenmods
```

Use `config.pkl.example` as the deployment starting point.
