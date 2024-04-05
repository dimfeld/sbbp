set -euxo pipefail

DOCKER=docker
if [ -n "$(which podman)" ]; then
  DOCKER=podman
fi

cd api
# clean is necessary for sqlx prepare to look at filigree crate, which uses sqlx macros
cargo clean -p filigree
cargo sqlx prepare

cd ../web

# Build with all dependencies
bun install --frozen-lockfile
PRECOMPRESS=true bun run build

# But get just the production dependencies to stick into the docker file
rm -rf node_modules
bun install --production --frozen-lockfile
# This isn't necessary for most cases but makes things work if you have symlinks to other places on disk,
# which I often do during development.
tar -c -z --dereference --uid=0 --gid=0 -f node_modules.tgz node_modules

cd ..
$DOCKER build -t ${TAG:-sbbp} -f docker/Dockerfile .

rm -f web/node_modules.tgz

