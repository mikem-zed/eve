ARG EVE_BUILDER_IMAGE=lfedge/eve-alpine:6.7.0
FROM ${EVE_BUILDER_IMAGE} AS build

ENV BUILD_PKGS curl build-base git 
ENV PKGS alpine-baselayout musl-utils bash

RUN eve-alpine-deploy.sh

ENV RUSTUP_HOME=/usr/local
ENV CARGO_HOME=/usr/local/cargo
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain none
ENV PATH "$PATH:/usr/local/cargo/bin"

RUN rustup default nightly
#RUN rustup override set 1.57.0

#cache crate registry
COPY Cargo.lock .
COPY local-reg ./local-reg
RUN mkdir .cargo && echo $'[source.crates-io] \n\
    registry = \'https://github.com/rust-lang/crates.io-index\' \n\
    replace-with = \'local-registry\' \n\
    [source.local-registry] \n\
    local-registry = \'/local-reg\'' > .cargo/config.toml

#and prebuild them
COPY Cargo.toml .
RUN mkdir src && touch src/lib.rs
RUN cargo build 
#--offline

FROM build as rust-builder
RUN rm src/lib.rs
COPY src ./src

RUN cargo install --debug --path .

FROM scratch as image
COPY --from=build /out /
ENV RUST_BACKTRACE full 
COPY --from=rust-builder /usr/local/cargo/bin/installer /sbin/installer
COPY run-installer.sh /sbin/run-installer.sh
COPY installer.sh .
COPY run-console.sh /

WORKDIR /
ENTRYPOINT []
CMD [/sbin/run-installer.sh]

