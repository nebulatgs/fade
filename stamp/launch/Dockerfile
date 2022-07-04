# syntax=docker/dockerfile:1.4
FROM rust:alpine
WORKDIR /app
RUN apk add --no-cache libc-dev
COPY --link . .
RUN --mount=type=cache,id=registry,target=/usr/local/cargo/registry --mount=type=cache,id=target,target=/app/target cargo fetch
RUN --mount=type=cache,id=registry,target=/usr/local/cargo/registry --mount=type=cache,id=target,target=/app/target cargo build --release
WORKDIR /target
RUN --mount=type=cache,id=target,target=/app/target cp /app/target/release/launch /target