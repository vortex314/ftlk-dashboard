set -x
WORKDIR="/work"
docker run -t --rm \
       -u "$(id -u):$(id -g)" \
       -e "HOME=${WORKDIR}"\
       -e "PKG_CONFIG_ALLOW_CROSS=1" \
       -w "${WORKDIR}" \
       -v "$(pwd):${WORKDIR}" \
       -v "${HOME}/.cargo/registry:/usr/local/cargo/registry" \
       mdirkse/rust_armv6:latest