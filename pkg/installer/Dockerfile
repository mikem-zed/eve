ARG EVE_BUILDER_IMAGE=lfedge/eve-alpine:6.7.0
FROM ${EVE_BUILDER_IMAGE} AS build

ENV BUILD_PKGS curl  build-base
# libgcc
#patch curl make gcc perl util-linux-dev git mtools linux-headers musl-dev xz-dev
# bash xorriso coreutils syslinux
RUN eve-alpine-deploy.sh

ENV RUSTUP_HOME=/usr/local/
ENV CARGO_HOME=/usr/local/
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain none
RUN rustup override set 1.57.0

FROM build as rust-builder
WORKDIR /usr/src/installer
COPY src ./src
COPY Cargo.toml ./
RUN cargo install --path .

FROM scratch as image
COPY --from=rust-builder /usr/local/bin/installer .

