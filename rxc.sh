WORKDIR="/work/workspace"
docker run -it --rm \
       -u "$(id -u):$(id -g)" \
       -e "HOME=${WORKDIR}" \
       -e "PKG_CONFIG_ALLOW_CROSS=1" \
       -w "${WORKDIR}" \
       -v "$(pwd):${WORKDIR}" \
       --name "rxc" \
       8d968a16895f /bin/bash
