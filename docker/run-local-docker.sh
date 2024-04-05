#!/bin/bash

DOCKER=docker
if [ -n "$(which podman)" ]; then
  DOCKER=podman
fi

source .env

DATABASE_URL=$(echo -n $DATABASE_URL | sed s/localhost/host.docker.internal/)

$DOCKER rm sbbp

set -e
$DOCKER run \
  -d \
  --name sbbp \
  -v ./storage:/storage \
  -v ./data:/data \
  -e TMPDIR=/storage/tmp \
  -e STORAGE_PROVIDER_DISK_LOCAL_BASE_PATH=/storage \
  -e DATABASE_URL=$DATABASE_URL \
  -e ANTHROPIC_API_KEY=$ANTHROPIC_API_KEY \
  -e DEEPGRAM_API_KEY=$DEEPGRAM_API_KEY \
  -p ${PORT:-8443}:8443 \
  sbbp
