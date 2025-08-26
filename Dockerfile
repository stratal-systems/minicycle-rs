FROM alpine:latest as base
WORKDIR /app

FROM base AS builder
COPY . .
RUN apk add cargo && cargo build --release

FROM base AS without-podman
RUN apk add git gnupg libgcc
COPY --from=builder /app/target/release/minicycle-rs /usr/local/bin/minicycle-rs
CMD "minicycle-rs"

FROM without-podman AS with-podman
RUN \
	apk add podman-remote && \
	ln -sf /usr/bin/podman-remote /usr/local/bin/podman

