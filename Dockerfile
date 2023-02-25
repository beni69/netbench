FROM rust as builder
ARG TARGETPLATFORM
ENV PROJECT=netbench
WORKDIR /src
RUN case ${TARGETPLATFORM} in \
    "linux/amd64") TARGET="x86_64-unknown-linux-musl" ;; \
    "linux/arm/v7") TARGET="armv7-unknown-linux-musleabihf" ;; \
    "linux/arm64") TARGET="aarch64-unknown-linux-musl" ;; \
    *) echo "unsupported platform: ${TARGETPLATFORM}" && exit 1 ;; \
    esac && \
    echo "TARGET=$TARGET" | tee /target && \
    rustup target add $TARGET

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && \
    . /target && \
    cargo build --release --target $TARGET && \
    ls -l target/$TARGET/release && \
    sleep 50 && \
    rm -rfv src target/$TARGET/release/deps/${PROJECT}*$

COPY src ./src
RUN . /target && \
    cargo build --release --target $TARGET && \
    cp -v target/$TARGET/release/${PROJECT} /app

CMD cat /target

FROM alpine
COPY --from=builder /app /app
