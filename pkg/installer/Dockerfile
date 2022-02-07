ARG EVE_BUILDER_IMAGE=lfedge/eve-alpine:6.7.0
#3b90b568b350b4134c71a67ebc5d845114c6c03d
FROM ${EVE_BUILDER_IMAGE} AS build

ENV BUILD_PKGS curl  build-base git
ENV PKGS util-linux 
#kbd
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
# RUN echo $'[http] \n\
#     multiplexing = false' > /usr/local/cargo/config

#RUN curl --proto '=https' --tlsv1.2 -sSf https://static.rust-lang.org/dist/channel-rust-nightly.toml.sha256

#RUN getent hosts static.rust-lang.org
#RUN nslookup static.rust-lang.org

RUN echo $'[net] \n\
    git-fetch-with-cli = true' > /usr/local/cargo/config
ENV RUSTUP_USE_CURL 0
ENV RUSTUP_USE_HYPER 1
RUN rustup default nightly
#RUN rustup override set 1.57.0

RUN mkdir src && touch src/lib.rs
COPY Cargo.lock .
COPY Cargo.toml .
#RUN cargo update -v
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
COPY --from=build /out /
ENV RUST_BACKTRACE full 
COPY --from=rust-builder /usr/local/cargo/bin/installer /sbin/installer
COPY run-installer.sh /sbin/run-installer.sh
#CMD [ "/bin/sh", "-c", "while true; do sleep 1; done" ]
#CMD ["openvt", "-s", "-v", "-w", "-e", "--", "/sbin/installer"]
CMD [/sbin/run-installer.sh]