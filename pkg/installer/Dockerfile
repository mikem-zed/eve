ARG EVE_BUILDER_IMAGE=lfedge/eve-alpine:6.7.0
FROM ${EVE_BUILDER_IMAGE} AS build

ENV BUILD_PKGS curl  build-base
# libgcc
#patch curl make gcc perl util-linux-dev git mtools linux-headers musl-dev xz-dev
# bash xorriso coreutils syslinux
RUN eve-alpine-deploy.sh

ENV RUSTUP_HOME=/usr/local
ENV CARGO_HOME=/usr/local/cargo
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain none
ENV PATH "$PATH:/usr/local/cargo/bin"


# it seems we are running buggy version of curl
# https://github.com/rust-lang/cargo/issues/7974
# https://www.reddit.com/r/rust/comments/ff5fs5/cratesio_timing_out/
RUN echo $'[http] \n\
multiplexing = false' > /usr/local/cargo/config

RUN rustup default nightly
#RUN rustup override set 1.57.0

RUN mkdir src && touch src/lib.rs
COPY Cargo.lock .
COPY Cargo.toml .
RUN cargo update -v
RUN cargo build
RUN cargo build --release

FROM build as rust-builder
RUN rm /src/lib.rs
#WORKDIR /usr/src/installer
COPY src ./src
#COPY Cargo.toml ./
#COPY Cargo.lock ./
RUN cargo install --debug --path .

FROM scratch as image
#COPY --from=build /out /
ENV RUST_BACKTRACE full 
COPY --from=rust-builder /usr/local/cargo/bin/installer /sbin/installer
CMD [ "/sbin/installer" ]

