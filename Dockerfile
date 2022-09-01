FROM rust:latest as builder
WORKDIR /home/andreas/git/webhook

RUN curl -fsSL https://get.docker.com | sh

RUN cargo init

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release
RUN rm ./src/*.rs

COPY ./src ./src

RUN rm ./target/release/deps/webhook*
RUN cargo build --release

COPY ./run ./run
COPY ./build ./build

COPY ./fullchain.pem ./fullchain.pem
COPY ./privkey.pem ./privkey.pem

CMD ["./run"]
