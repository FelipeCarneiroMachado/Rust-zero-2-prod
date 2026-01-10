# Some setup
FROM lukemathwalker/cargo-chef:latest-rust-1.92.0 as chef
WORKDIR /app
RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install musl-tools lld clang -y

# Something something pre compile deps
FROM chef as planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json


FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
# Uo to here everything should be cached

# Now we build
COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release --target x86_64-unknown-linux-musl

# Runtime stage
FROM alpine:latest AS runtime
WORKDIR /app
# Install with apk since we're using alpine
RUN apk add --no-cache libgcc openssl ca-certificates
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/zero2prod zero2prod
COPY configuration configuration
ENV APP_ENVIRONMENT production
ENTRYPOINT ["./zero2prod"]