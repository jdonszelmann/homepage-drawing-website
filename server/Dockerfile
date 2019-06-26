FROM rust:latest
RUN USER=root cargo new drawserver
WORKDIR /drawserver

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build +nightly --release
RUN rm src/*.rs

COPY ./src ./src

RUN rm ./target/release/deps/drawserver*
RUN cargo build +nightly --release

EXPOSE 80
CMD ["./target/release/drawserver"]
#CMD ["sleep","10000"]
