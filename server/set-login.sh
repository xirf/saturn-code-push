#!/usr/bin/env bash
set -euo pipefail

server_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
env_file="$server_dir/.env"
engine=${CONTAINER_ENGINE:-podman}
username=${1:-admin}

if [[ ! -f "$env_file" ]]; then
    printf '%s\n' 'Run sh server/init-env.sh first.' >&2
    exit 1
fi

if [[ ! "$username" =~ ^[a-zA-Z0-9_.@-]{1,128}$ ]]; then
    printf '%s\n' 'Use a username of 1 to 128 letters, digits, dots, underscores, @ signs, or hyphens.' >&2
    exit 1
fi

IFS= read -r -s -p 'New password (12 to 1024 bytes): ' password
printf '\n'
IFS= read -r -s -p 'Confirm password: ' confirmation
printf '\n'

export LC_ALL=C
if [[ "$password" != "$confirmation" || ${#password} -lt 12 || ${#password} -gt 1024 ]]; then
    printf '%s\n' 'Passwords must match and contain 12 to 1024 bytes.' >&2
    exit 1
fi

# Pass the password on stdin so it never appears in process arguments or the environment.
hash=$(printf '%s' "$password" | "$engine" compose \
    --env-file "$env_file" -f "$server_dir/compose.yaml" \
    run --rm --no-deps -T code-push saturn-code-push --hash-password)
unset password confirmation

if [[ "$hash" != '$argon2id$'* ]]; then
    printf '%s\n' 'Password hashing failed; build the server image first.' >&2
    exit 1
fi

umask 077
temporary=$(mktemp "$server_dir/.env.XXXXXX")
trap 'rm -f -- "$temporary"' EXIT
awk '!/^ADMIN_(USERNAME|PASSWORD_HASH)=/' "$env_file" > "$temporary"
printf "\nADMIN_USERNAME=%s\nADMIN_PASSWORD_HASH='%s'\n" "$username" "$hash" >> "$temporary"
mv -- "$temporary" "$env_file"
unset hash

"$engine" compose --env-file "$env_file" -f "$server_dir/compose.yaml" \
    up -d code-push
printf '%s\n' 'Bootstrap credentials configured. Existing accounts are unchanged; manage passwords in Settings.'
