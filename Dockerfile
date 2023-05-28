FROM ghcr.io/cross-rs/arm-unknown-linux-gnueabi:latest
ARG DEBIAN_FRONTEND=noninteractive

RUN dpkg --add-architecture armel && \
    apt-get update && \
    apt-get install --assume-yes --no-install-recommends \
    doxygen graphviz latexmk texlive-base \
    libx11-dev:armel libxext-dev:armel libxft-dev:armel \
    libxinerama-dev:armel libxcursor-dev:armel \
    libxrender-dev:armel  libxfixes-dev:armel  libgl1-mesa-dev:armel \
    libglu1-mesa-dev:armel libasound2-dev:armel libpango1.0-dev:armel 

ENV  CROSS_CMAKE_OBJECT_FLAGS="-ffunction-sections -fdata-sections -fPIC -march=armv6 -marm -mfloat-abi=soft -L/usr/lib/arm-linux-gnueabihf/"

