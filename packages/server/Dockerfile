# Use an official Rust runtime as a parent image
FROM rust:1.73

# Set the working directory in the container to /usr/src/myapp
WORKDIR /usr/biasdo-server

# Copy the current directory contents into the container at /usr/src/myapp
COPY . .

RUN --mount=type=secret,id=DATABASE_URL \
    echo "DATABASE_URL=$(cat /run/secrets/DATABASE_URL)" > .env

RUN cargo install sqlx-cli
RUN cargo sqlx migrate run
RUN cargo sqlx prepare

RUN cargo install --path .

CMD ["biasdo_server"]
