set -x
clear
source ~/.cargo/env
export CC=arm-linux-gnueabihf-gcc 
export CXX=arm-linux-gnueabihf-g++ 
export CARGO_TARGET_ARM_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc 
export CC_arm_unknown_linux_gnueabihf=arm-linux-gnueabi-gcc 
export CXX_arm_unknown_linux_gnueabihf=arm-linux-gnueabi-g++ 
export PKG_CONFIG_PATH="/usr/lib/arm-linux-gnueabi/pkgconfig/:${PKG_CONFIG_PATH}"
# docker pull dockcross/linux-armv6:latest
# docker pull ghcr.io/cross-rs/arm-unknown-linux-gnueabi:latest
cargo build --target=arm-unknown-linux-gnueabihf --release --verbose
