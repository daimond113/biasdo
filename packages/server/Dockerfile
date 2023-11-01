# Use an official Rust runtime as a parent image
FROM rust:1.73

ARG DATABASE_URL

# Set the working directory in the container to /usr/src/myapp
WORKDIR /usr/biasdo-server

# Copy the current directory contents into the container at /usr/src/myapp
COPY . .

RUN cargo install --path .

EXPOSE 443

RUN cargo install sqlx-cli
RUN cargo sqlx prepare

CMD ["biasdo_server"]
