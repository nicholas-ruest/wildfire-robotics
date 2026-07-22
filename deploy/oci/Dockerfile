# syntax=docker/dockerfile:1.7
FROM rust:1.97.1-bookworm@sha256:77fac8b98f9f46062bb680b6d25d5bcaabfc400143952ebc572e924bcbedc3fa AS builder
WORKDIR /src
ARG PACKAGE
ARG BINARY
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    test -n "$PACKAGE" && test -n "$BINARY" && \
    cargo build --locked --release -p "$PACKAGE" --bin "$BINARY" && \
    install -D -m 0555 "target/release/$BINARY" /out/service

FROM gcr.io/distroless/cc-debian12:nonroot@sha256:fccdbb0a547c14e23fcf4ce8ad62ca5d43b4faae8d22cd292f490fef9946c96e
COPY --from=builder --chown=65532:65532 /out/service /usr/local/bin/service
USER 65532:65532
ENTRYPOINT ["/usr/local/bin/service"]
