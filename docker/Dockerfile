# This Dockerfile builds both the web application and the API server into a single container,
# ans sets up the API server to forward non-API requests to the web application.

##### API Builders

FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
RUN apt-get update && apt-get install -y pkg-config libssl-dev
WORKDIR /app

FROM chef AS planner
COPY api/ .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS api-builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY api/ .
RUN cargo build --release --bin sbbp-api



##### Web Builders

### Base Web Image
FROM oven/bun:1.1.0-slim AS web-base
WORKDIR /app

### Dependency installer image
FROM web-base AS web-install
# Install both dev and prod dependencies so we can build with the former but only use the latter in the final image
RUN mkdir -p /tmp/dev
COPY web/package.json web/bun.lockb /tmp/dev/
RUN cd /tmp/dev && bun install --frozen-lockfile

RUN mkdir -p /tmp/prod
COPY web/package.json web/bun.lockb /tmp/prod/
RUN cd /tmp/prod && bun install --production --frozen-lockfile



### Web builder image
FROM web-base AS web-builder
ENV NODE_ENV=production
COPY --from=web-install /tmp/dev/node_modules node_modules
COPY web/ .
RUN bun run build:production



##### Final image
FROM web-base AS runtime
ARG TARGETARCH

RUN mkdir -p /data
RUN apt-get update && apt-get install -y dumb-init pkg-config libssl-dev ca-certificates ffmpeg python3 && apt-get clean
RUN update-ca-certificates

ADD https://github.com/DarthSim/hivemind/releases/download/v1.1.0/hivemind-v1.1.0-linux-$TARGETARCH.gz /tmp/hivemind.gz
RUN gunzip /tmp/hivemind.gz && mv /tmp/hivemind /usr/local/bin/
ADD docker/Procfile /app

RUN wget https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -O /usr/local/bin/yt-dlp && chmod +x /usr/local/bin/yt-dlp
COPY --from=api-builder /app/target/release/sbbp-api /app/api
COPY --from=web-install /tmp/prod/node_modules /app/web/node_modules
COPY --from=web-builder /app/build /app/web
COPY --from=web-builder /app/package.json /app/web

ENV HOST=::0
ENV ENV=production
ENV INSECURE=false

# Primary server port
ENV PORT=8443
EXPOSE 8443/tcp

# This isn't exposed, but sets up the frontend to listen on this port internally and the API to forward
# requests to it. If you change this value you should change the corresponding PORT setting in Procfile.
ENV FRONTEND_PORT=5173
ENV FRONTEND_ASSET_DIR=/app/web/client
ENV SERVE_FRONTEND=true

# Port to listen on HTTP, which just redirects to HTTPS
ENV HTTP_PORT=8080
EXPOSE 8080/tcp

ENV SUPERVISOR_LOGLEVEL=info

ENV ALLOW_PUBLIC_SIGNUP=false
ENV ALLOW_INVITE_TO_SAME_ORG=false
ENV ALLOW_INVITE_TO_NEW_ORG=false

ENV QUEUE_PATH=/data/queue.db

# [dumb-init](https://github.com/Yelp/dumb-init) for proper PID1 handling
ENTRYPOINT [ "/usr/bin/dumb-init" ]
# [hivemind](https://github.com/DarthSim/hivemind) is a very simple process manager that will start both processes and shut down everything if either of them stop.
# We leave the rest of the process management to Docker itself
CMD [ "/usr/local/bin/hivemind", "/app/Procfile" ]
