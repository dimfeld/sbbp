set -euxo pipefail

DOCKER=docker
if [ -n "$(which podman)" ]; then
  DOCKER=podman
fi

cd api
# clean is necessary for sqlx prepare to look at filigree crate, which uses sqlx macros
cargo clean -p filigree
cargo sqlx prepare

cd ..
$DOCKER build -t ${TAG:-sbbp} -f docker/Dockerfile .

