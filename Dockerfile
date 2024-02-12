FROM lukemathwalker/cargo-chef:latest-rust-slim-buster AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin test-app

FROM gcr.io/distroless/cc-debian12 as runtime
WORKDIR /app
COPY --from=builder /app/target/release/test-app test-app
ENV RUST_LOG trace
ENTRYPOINT ["./test-app"]
