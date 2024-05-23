FROM rust:1.77-bullseye AS builder

WORKDIR /usr/src/backend

COPY . .

RUN cargo build --release

FROM debian:bullseye-slim

COPY --from=builder /usr/src/backend/target/release/backend /usr/local/bin/

CMD ["/usr/local/bin/backend"]
