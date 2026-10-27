FROM node:26-alpine AS console
WORKDIR /console
COPY console/package.json console/package-lock.json ./
RUN --mount=type=cache,target=/root/.npm npm ci
COPY console/ ./
COPY README.md LICENSE NOTICE CONTRIBUTING.md SECURITY.md THIRD_PARTY_NOTICES.md /
COPY docs/ /docs/
COPY server/README.md server/ARCHITECTURE.md /server/
COPY packages/saturn_assets/README.md /packages/saturn_assets/README.md
RUN npx --no-install nuxt prepare && npm run check && npm run build

FROM rust:1.96-bookworm AS development
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev && rm -rf /var/lib/apt/lists/* \
    && rustup component add rustfmt clippy
WORKDIR /workspace
COPY server/Cargo.toml server/Cargo.lock ./
COPY server/src ./src
COPY server/migrations ./migrations
CMD ["cargo", "test", "--locked"]

FROM development AS builder
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/workspace/target \
    cargo build --release --locked && cp target/release/saturn-code-push /workspace/saturn-code-push

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends libssl3 ca-certificates && rm -rf /var/lib/apt/lists/* \
    && groupadd --gid 10001 saturn && useradd --uid 10001 --gid saturn saturn \
    && mkdir /data && chown saturn:saturn /data
COPY --from=builder /workspace/saturn-code-push /usr/local/bin/saturn-code-push
COPY --from=console /console/.output/public /app/console
COPY LICENSE NOTICE THIRD_PARTY_NOTICES.md /usr/share/doc/saturn-code-push/
USER saturn
ENV DATA_DIR=/data RUST_LOG=info CONSOLE_DIR=/app/console
EXPOSE 3000
CMD ["saturn-code-push"]
