#!/usr/bin/env bash
set -euo pipefail

repository=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
engine=${CONTAINER_ENGINE:-docker}
network="saturn-console-e2e-$$"
server="saturn-console-e2e-server-$$"
application_image=${E2E_APPLICATION_IMAGE:-saturn-console-e2e-app:local}
browser_image=${E2E_BROWSER_IMAGE:-saturn-console-e2e-browser:local}
token="sb_api_$(openssl rand -hex 32)"
password=$(openssl rand -hex 24)

cleanup() {
    "$engine" rm -f "$server" >/dev/null 2>&1 || true
    "$engine" network rm "$network" >/dev/null 2>&1 || true
}
trap cleanup EXIT

"$engine" build -f "$repository/server/Dockerfile" -t "$application_image" "$repository"
"$engine" build -f "$repository/console/Dockerfile.e2e" -t "$browser_image" "$repository"
hash=$(printf '%s' "$password" | "$engine" run --rm -i "$application_image" saturn-code-push --hash-password)
"$engine" network create "$network" >/dev/null
"$engine" run -d --name "$server" --network "$network" --network-alias saturn-e2e-server \
    --read-only --cap-drop ALL --security-opt no-new-privileges:true \
    --tmpfs /data:rw,uid=10001,gid=10001,mode=0700 \
    -e "ADMIN_TOKEN=$token" \
    -e PUBLIC_BASE_URL=http://saturn-e2e-server:3000 \
    -e ADMIN_USERNAME=admin \
    -e "ADMIN_PASSWORD_HASH=$hash" \
    "$application_image" >/dev/null

mkdir -p "$repository/console/test-results"
mkdir -p "$repository/console/playwright-report"
"$engine" run --rm --network "$network" --shm-size=1g \
    -e "E2E_ADMIN_TOKEN=$token" \
    -e "E2E_PASSWORD=$password" \
    -v "$repository/console/test-results:/runner/test-results:Z" \
    -v "$repository/console/playwright-report:/runner/playwright-report:Z" \
    "$browser_image"
