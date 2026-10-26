FROM node:26-alpine AS console
WORKDIR /console
COPY console/package.json console/package-lock.json ./
RUN --mount=type=cache,target=/root/.npm npm ci
COPY console/ ./
RUN npm run build

FROM rust:1.96-bookworm AS builder
WORKDIR /build
COPY server/Cargo.toml server/Cargo.lock ./
COPY server/src ./src
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/build/target \
    cargo build --release --locked && cp target/release/saturn-code-push /build/saturn-code-push

FROM debian:bookworm-slim
RUN groupadd --gid 10001 saturn && useradd --uid 10001 --gid saturn saturn \
    && mkdir /data && chown saturn:saturn /data
COPY --from=builder /build/saturn-code-push /usr/local/bin/saturn-code-push
COPY --from=console /console/dist /app/console
USER saturn
ENV DATA_DIR=/data RUST_LOG=info CONSOLE_DIR=/app/console
EXPOSE 3000
CMD ["saturn-code-push"]
