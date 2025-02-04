FROM ubuntu:25.04 AS builder

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update
RUN apt-get install -y curl build-essential libssl-dev pkg-config

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /usr/local/backend
COPY . .

RUN cargo build --release

FROM ubuntu:25.04

COPY --from=builder /usr/local/backend/target/release/backend /usr/local/bin/

CMD [ "/usr/local/bin/backend" ]