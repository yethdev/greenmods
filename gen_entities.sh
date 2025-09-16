#!/bin/bash

sea migrate up -d crates/modhost-migrations

sea generate entity \
    -o crates/modhost-entities/src \
    --with-serde both \
    --with-copy-enums \
    --serde-skip-deserializing-primary-key \
    --with-prelude all-allow-unused-imports \
    --compact-format \
    --model-extra-derives utoipa::ToSchema,utoipa::ToResponse,Hash \
    --enum-extra-derives utoipa::ToSchema,utoipa::ToResponse,Hash,strum_macros::Display,strum_macros::EnumString \
    --lib \
    "$@"
