#!/usr/bin/env sh
set -eu

server_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
env_file="$server_dir/.env"

if [ -e "$env_file" ]; then
    printf '%s\n' 'server/.env already exists; no changes made.'
    exit 0
fi

umask 077
setup_token=$(openssl rand -hex 32)
postgres_password=$(openssl rand -hex 32)

# Noclobber also prevents a concurrent setup process from replacing this file.
(
    set -C
    {
        printf 'SETUP_TOKEN=%s\n' "$setup_token"
        printf 'POSTGRES_PASSWORD=%s\n' "$postgres_password"
        printf 'DATABASE_URL=postgresql://saturn:%s@postgres:5432/saturn\n' "$postgres_password"
        printf '%s\n' \
            'PUBLIC_BASE_URL=http://localhost:3000' \
            'HOST_PORT=3000' \
            'RUST_LOG=info'
    } > "$env_file"
)

printf '%s\n' 'Created server/.env with random PostgreSQL/setup credentials and owner-only permissions.'
