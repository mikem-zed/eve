ARG EVE_BUILDER_IMAGE=lfedge/eve-alpine:6.7.0
FROM ${EVE_BUILDER_IMAGE} AS build

ENV BUILD_PKGS curl build-base git 
#ncurses-dev
ENV PKGS alpine-baselayout musl-utils smartmontools bash mtools util-linux e2fsprogs e2fsprogs-extra

RUN eve-alpine-deploy.sh

# setup rust
ENV RUSTUP_HOME=/usr/local
ENV CARGO_HOME=/usr/local/cargo
# get rustup first
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain none
ENV PATH "$PATH:/usr/local/cargo/bin"
# and install selected version
COPY rust-toolchain.toml .
COPY .cargo .
# the following command must go away after 
# https://github.com/rust-lang/rustup/issues/2686 is fixed
# currently there is no way to setup ONLY version form rust-toolchain.toml
# so we install a minimal toolchain to make cargo happy
# cargo wll install necessary toolchain before building
RUN rustup set profile minimal
RUN rustup default stable 

#cache crate registry
COPY Cargo.lock .
COPY local-reg ./local-reg
# for some reason _sometimes_ cargo cannot download dependencies when running
# inside a container so download them in a Makefile beforehand
# and override a registry with our local copy
RUN mkdir .cargo && echo $'[source.crates-io] \n\
    registry = \'https://github.com/rust-lang/crates.io-index\' \n\
    replace-with = \'local-registry\' \n\
    [source.local-registry] \n\
    local-registry = \'/local-reg\'' >> .cargo/config.toml

# cursive has a way to fine tune backend to use stdio but
# I couldn't figure out how yet. this patch should eventually go away
RUN git clone https://github.com/gyscos/cursive.git 
WORKDIR /cursive
RUN git checkout 3eada33c049c98f04ec00ee92b6c849d68cd322a
COPY patch/cursive/* .
RUN git apply *.patch
WORKDIR /

#and prebuild everything
COPY Cargo.toml .
RUN mkdir src && touch src/lib.rs
#COPY cursive ./cursive
RUN cargo build --offline

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
COPY grub.cfg.in /

WORKDIR /
ENTRYPOINT []
CMD [/sbin/run-installer.sh]

