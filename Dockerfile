FROM dockcross/linux-armv6 AS ubuntu_build

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update -qq
RUN apt-get install -y --no-install-recommends lsb-release g++-arm-linux-gnueabihf g++ cmake curl tar git make
RUN apt-get install -y ca-certificates && update-ca-certificates --fresh && export SSL_CERT_DIR=/etc/ssl/certs
RUN dpkg --add-architecture armhf 
RUN sed -i "s/deb http/deb [arch=amd64] http/" /etc/apt/sources.list
RUN echo "deb [arch=armhf] http://ports.ubuntu.com/ $(lsb_release -c -s) main multiverse universe" | tee -a /etc/apt/sources.list 
RUN echo "deb [arch=armhf] http://ports.ubuntu.com/ $(lsb_release -c -s)-security main multiverse universe" | tee -a /etc/apt/sources.list 
RUN echo "deb [arch=armhf] http://ports.ubuntu.com/ $(lsb_release -c -s)-backports main multiverse universe" | tee -a /etc/apt/sources.list 
RUN echo "deb [arch=armhf] http://ports.ubuntu.com/ $(lsb_release -c -s)-updates main multiverse universe" | tee -a /etc/apt/sources.list 
RUN apt-get update -qq && apt-get install -y --no-install-recommends -o APT::Immediate-Configure=0 libx11-dev:armhf libxext-dev:armhf libxft-dev:armhf libxinerama-dev:armhf libxcursor-dev:armhf libxrender-dev:armhf libxfixes-dev:armhf libpango1.0-dev:armhf libgl1-mesa-dev:armhf libglu1-mesa-dev:armhf libasound2-dev:armhf
RUN curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain stable --profile minimal -y

ENV PATH="/root/.cargo/bin:$PATH" \
CC=arm-linux-gnueabihf-gcc CXX=arm-linux-gnueabihf-g++ \
	CARGO_TARGET_ARM_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc \
    CC_arm_unknown_linux_gnueabi=arm-linux-gnueabihf-gcc \
    CXX_arm_unknown_linux_gnueabi=arm-linux-gnueabihf-g++ \
    PKG_CONFIG_PATH="/usr/lib/arm-linux-gnueabihf/pkgconfig/:${PKG_CONFIG_PATH}"

RUN rustup target add arm-unknown-linux-gnueabihf

COPY . .

RUN  cargo build --release --target=arm-unknown-linux-gnueabihf

FROM scratch AS export-stage
COPY --from=ubuntu_build target/arm-unknown-linux-gnueabihf/release/fl fl.pi