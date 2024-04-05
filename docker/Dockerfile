# This Dockerfile builds both the web application and the API server into a single container,
# ans sets up the API server to forward non-API requests to the web application.


#### Base Runtime Image
FROM debian:bookworm-slim AS bun-base
ARG BUN_VERSION=1.1.1
WORKDIR /app
ARG TARGETARCH
RUN apt-get update && apt-get install -y dumb-init pkg-config libssl-dev ca-certificates && apt-get clean
RUN update-ca-certificates

# default bun image is on an old version of Debian so we just copy Bun from it
COPY --from=oven/bun:${BUN_VERSION}-slim /usr/local/bin/bun /usr/local/bin/bun
RUN ln -s /usr/local/bin/bun /usr/local/bin/bunx


##### API Builders

FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
ENV SQLX_OFFLINE=true
ENV SQLX_OFFLINE_DIR=/app/.sqlx
RUN apt-get update && apt-get install -y pkg-config libssl-dev
WORKDIR /app

FROM chef AS planner
COPY api/ .
ENV SQLX_OFFLINE=true
# Make sure sqlx always looks in this directory, even when building other crates.
ENV SQLX_OFFLINE_DIR=/app/.sqlx
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS api-builder
COPY --from=planner /app/recipe.json recipe.json
COPY api/.sqlx ./.sqlx/
RUN cargo chef cook --release --recipe-path recipe.json
COPY api/ .
RUN cargo build --release --bin sbbp-api



##### Web Builders

# It's much simpler to build the web application outside of Docker.
# As of April 2024 I was also seeing a lot of node-gyp issues installing with Bun inside the container.

# ### Base Web Image
# FROM oven/bun:${BUN_VERSION} AS web-base
# WORKDIR /app

# ### Dependency installer image
# FROM web-base AS web-install
# # node-gyp dependencies
# RUN apt-get update && apt-get -y install python3
# # Install both dev and prod dependencies so we can build with the former but only use the latter in the final image
# RUN mkdir -p /tmp/dev
# COPY web/package.json web/bun.lockb /tmp/dev/
# # First one may fail due to node-gyp problems that only matter for Vite dev
# RUN cd /tmp/dev && bun install --frozen-lockfile && true
# RUN cd /tmp/dev && bun install --frozen-lockfile

# RUN mkdir -p /tmp/prod
# COPY web/package.json web/bun.lockb /tmp/prod/
# # First one may fail due to node-gyp problems that only matter for Vite dev
# RUN cd /tmp/prod && bun install --production --frozen-lockfile && true
# RUN cd /tmp/prod && bun install --production --frozen-lockfile



# ### Web builder image
# FROM web-base AS web-builder
# ENV NODE_ENV=production
# COPY --from=web-install /tmp/dev/node_modules node_modules
# COPY web/ .
# RUN PRECOMPRESS=true bun run build



##### Final image
FROM bun-base AS runtime
WORKDIR /app
ARG TARGETARCH

RUN mkdir -p /data
RUN apt-get install -y curl ffmpeg python3 && apt-get clean

ADD https://github.com/DarthSim/hivemind/releases/download/v1.1.0/hivemind-v1.1.0-linux-$TARGETARCH.gz /tmp/hivemind.gz
RUN gunzip /tmp/hivemind.gz && mv /tmp/hivemind /usr/local/bin/ && chmod +x /usr/local/bin/hivemind

RUN curl -L https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -o /usr/local/bin/yt-dlp && chmod +x /usr/local/bin/yt-dlp

ADD docker/Procfile /app
COPY --from=api-builder /app/target/release/sbbp-api /app/api

# Copy the whole app from context since we're not using builder images for web right now.
# Use the node_modules.tgz created by build-docker.sh. Only required if you have symlinks to places outside the project
# in your node_modules; if not you can just add the node_modules directory.
ADD ./web/node_modules.tgz /app/web/
COPY ./web/package.json /app/web/
COPY ./web/build/ /app/web/

# The right method if you're using the builder containers above.
# COPY --from=web-install /tmp/prod/node_modules /app/web/node_modules
# COPY --from=web-builder /app/build /app/web
# COPY --from=web-builder /app/package.json /app/web

ENV HOST=::0
ENV ENV=production
ENV INSECURE=false

# Primary server port
ENV PORT=8443
EXPOSE 8443/tcp
ENV API_SERVER=localhost:8443

# This isn't exposed, but sets up the frontend to listen on this port internally and the API to forward
# requests to it. If you change this value you should change the corresponding PORT setting in Procfile.
ENV FRONTEND_PORT=5173
ENV FRONTEND_ASSET_DIR=/app/web/client
ENV SERVE_FRONTEND=true

# Port to listen on HTTP, which just redirects to HTTPS
ENV HTTP_PORT=8080
EXPOSE 8080/tcp

ENV ALLOW_PUBLIC_SIGNUP=false
ENV ALLOW_INVITE_TO_SAME_ORG=false
ENV ALLOW_INVITE_TO_NEW_ORG=false

ENV QUEUE_PATH=/data/queue.db

# [dumb-init](https://github.com/Yelp/dumb-init) for proper PID1 handling
ENTRYPOINT [ "/usr/bin/dumb-init" ]
# [hivemind](https://github.com/DarthSim/hivemind) is a very simple process manager that will start both processes and shut down everything if either of them stop.
# We leave the rest of the process management to Docker itself
CMD [ "/usr/local/bin/hivemind", "/app/Procfile" ]
