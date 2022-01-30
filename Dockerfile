FROM rust:latest AS builder

RUN rustup target add x86_64-unknown-linux-musl

RUN apt update && apt install -y musl-tools musl-dev

RUN update-ca-certificates

WORKDIR /build

COPY ./ .

#RUN cargo build --release
RUN cargo build --target x86_64-unknown-linux-musl --release


FROM busybox

WORKDIR /mangaworld

COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/mangaworld_api ./

RUN mkdir /manga && mkdir /manga/download && mkdir /manga/state && chmod -R 777 /manga

CMD ["/mangaworld/mangaworld_api"]