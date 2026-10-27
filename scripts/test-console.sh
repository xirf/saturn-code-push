#!/usr/bin/env bash
set -euo pipefail

repository=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
engine=${CONTAINER_ENGINE:-docker}
network="saturn-console-e2e-$$"
server="saturn-console-e2e-server-$$"
database="saturn-console-e2e-postgres-$$"
application_image=${E2E_APPLICATION_IMAGE:-saturn-console-e2e-app:local}
browser_image=${E2E_BROWSER_IMAGE:-saturn-console-e2e-browser:local}
token="sb_api_$(openssl rand -hex 32)"
password=$(openssl rand -hex 24)
setup_token=$(openssl rand -hex 32)

cleanup() {
    "$engine" rm -f "$server" >/dev/null 2>&1 || true
    "$engine" rm -f "$database" >/dev/null 2>&1 || true
    "$engine" network rm "$network" >/dev/null 2>&1 || true
}
trap cleanup EXIT

"$engine" build -f "$repository/server/Dockerfile" -t "$application_image" "$repository"
"$engine" build -f "$repository/console/Dockerfile.e2e" -t "$browser_image" "$repository"
"$engine" network create "$network" >/dev/null
"$engine" run -d --name "$database" --network "$network" --network-alias saturn-e2e-postgres \
    -e POSTGRES_USER=saturn -e POSTGRES_PASSWORD=e2e-only -e POSTGRES_DB=saturn \
    --tmpfs /var/lib/postgresql/data postgres:17-alpine >/dev/null
for attempt in $(seq 1 30); do
    if "$engine" exec "$database" pg_isready -U saturn -d saturn >/dev/null 2>&1; then break; fi
    sleep 1
done
"$engine" exec "$database" pg_isready -U saturn -d saturn >/dev/null
"$engine" run -d --name "$server" --network "$network" --network-alias saturn-e2e-server \
    --read-only --cap-drop ALL --security-opt no-new-privileges:true \
    --tmpfs /data:rw,uid=10001,gid=10001,mode=0700 \
    -e "ADMIN_TOKEN=$token" \
    -e DATABASE_URL=postgresql://saturn:e2e-only@saturn-e2e-postgres/saturn \
    -e PUBLIC_BASE_URL=http://saturn-e2e-server:3000 \
    -e "SETUP_TOKEN=$setup_token" \
    "$application_image" >/dev/null

mkdir -p "$repository/console/test-results"
mkdir -p "$repository/console/playwright-report"
"$engine" run --rm --network "$network" --shm-size=1g \
    -e "E2E_ADMIN_TOKEN=$token" \
    -e "E2E_PASSWORD=$password" \
    -e "E2E_SETUP_TOKEN=$setup_token" \
    -v "$repository/console/test-results:/runner/test-results:Z" \
    -v "$repository/console/playwright-report:/runner/playwright-report:Z" \
    "$browser_image"
