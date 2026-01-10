FROM rust:1.92.0
WORKDIR APP
RUN apt update && apt install lld clang -
COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release
ENTRYPOINT ["./target/release/zero2prod"]