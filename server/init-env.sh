#!/usr/bin/env sh
set -eu

server_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
env_file="$server_dir/.env"

if [ -e "$env_file" ]; then
    printf '%s\n' 'server/.env already exists; no changes made.'
    exit 0
fi

umask 077
token=$(openssl rand -hex 32)

# Noclobber also prevents a concurrent setup process from replacing this file.
(
    set -C
    {
        printf 'ADMIN_TOKEN=sb_api_%s\n' "$token"
        printf '%s\n' \
            'PUBLIC_BASE_URL=http://localhost:3000' \
            'HOST_PORT=3000' \
            'RUST_LOG=info' \
            'ADMIN_USERNAME=admin'
    } > "$env_file"
)

printf '%s\n' 'Created server/.env with a random API token and owner-only permissions.'
