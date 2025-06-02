FROM rust:1.76 as builder

WORKDIR /app
COPY . .

RUN cargo build --release

FROM debian:bullseye-slim

COPY --from=builder /app/target/release/{{project_name}} /app/{{project_name}}

WORKDIR /app
CMD ["./{{project_name}}"]
