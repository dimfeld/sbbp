### API Builders

FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
RUN apt-get update && apt-get install -y pkg-config libssl-dev
WORKDIR /app

FROM chef AS planner
COPY api/ .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS api-builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY api/ .
RUN cargo build --release --bin sbbp-api



### Web Builders
FROM oven/bun:1.1.0-slim AS web-base
WORKDIR /app

FROM web-base AS web-install
# Install dev
RUN mkdir -p /tmp/dev
COPY package.json bun.lockb /tmp/dev/
RUN cd /tmp/dev && bun install --frozen-lockfile
# Install prod
RUN mkdir -p /tmp/prod
COPY package.json bun.lockb /tmp/prod/
RUN cd /tmp/prod && bun install --production --frozen-lockfile


FROM web-base AS web-builder
ENV NODE_ENV=production
COPY --from=web-install /tmp/dev/node_modules node_modules
COPY web/ .
RUN bun run build



### Final image
FROM web-base AS runtime
RUN apt-get update && apt-get install -y pkg-config libssl-dev ca-certificates
RUN update-ca-certificates
COPY --from=api-builder /app/target/release/sbbp-api /usr/local/bin
COPY --from=web-install /tmp/prod/node_modules node_modules
COPY --from=web-builder /app/build /app/web

EXPOSE 8080/tcp
ENV PORT=8080
ENV ENV=production
ENV QUEUE_PATH=/data/queue.db
ENV HOST=0.0.0.0
ENV INSECURE=false
ENV ALLOW_PUBLIC_SIGNUP=false
ENV ALLOW_INVITE_TO_SAME_ORG=false
ENV ALLOW_INVITE_TO_NEW_ORG=false

ENTRYPOINT ["/usr/local/bin/sbbp-api"]
