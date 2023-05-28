set -x
clear
unset all_proxy 
unset http_proxy 
unset https_proxy 
unset no_proxy
set PKG_CONFIG_PATH="/usr/lib/arm-linux-gnueabihf/pkgconfig/:${PKG_CONFIG_PATH}"
# docker pull dockcross/linux-armv6:latest
# docker pull ghcr.io/cross-rs/arm-unknown-linux-gnueabi:latest
cross build --target=arm-unknown-linux-gnueabihf --release
