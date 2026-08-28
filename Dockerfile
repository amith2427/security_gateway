# Build stage
FROM rust:slim AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

# Run stage
FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/security_gateway .
EXPOSE 8082
CMD ["./security_gateway"]