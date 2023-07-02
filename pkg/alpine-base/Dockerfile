FROM busybox@sha256:ef320ff10026a50cf5f0213d35537ce0041ac1d96e9b7800bafd8bc9eff6c693 AS builder

ENV ALPINE_VERSION=3.18
ENV MINI_ROOTFS_VERSION=${ALPINE_VERSION}.2

WORKDIR /rootfs

# Since riscv64 is not officially supported yet, we use distinct package source
# busybox do not support https to storage.googleapis.com so we use http here
# and replace with https for repo url
RUN case $(uname -m) in \
    x86_64|aarch64) \
        ROOTFS_URL="https://dl-cdn.alpinelinux.org/alpine/v$ALPINE_VERSION/releases/$(uname -m)/alpine-minirootfs-$MINI_ROOTFS_VERSION-$(uname -m).tar.gz"; \
        ;; \
    riscv64) \
        ROOT_URL=http://storage.googleapis.com/eve-alpine && \
        ROOTFS_URL="$ROOT_URL/images/alpine-minirootfs-220909-riscv64.tar.gz"; \
        ;; \
    *) echo "Unsupported architecture $(uname -m). Exiting" && exit 1 \
      ;; \
    esac && \
    wget -q -O /tmp/rootfs.tar.gz "$ROOTFS_URL" && \
    tar xzf /tmp/rootfs.tar.gz && \
    if [ "$(uname -m)" = "riscv64" ] ; then \
        wget -q -P etc/apk/keys \
            "$ROOT_URL/keys/builder@projecteve.dev-62fe6660.rsa.pub" && \
        ROOT_URL=https://storage.googleapis.com/eve-alpine && \
        echo "$ROOT_URL/v$(ALPINE_VERSION)/main" > etc/apk/repositories && \
        echo "$ROOT_URL/v$(ALPINE_VERSION)/community" >> etc/apk/repositories && \
        echo "$ROOT_URL" > etc/apk/cache.url; \
    fi

FROM scratch

# seed the root filesystem
COPY --from=builder /rootfs/ /

# set the defaults for docker run
ENV PATH /usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin
CMD sh
