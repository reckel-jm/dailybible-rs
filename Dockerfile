FROM rust:1.85.0-slim-bookworm as builder

# Create new build dir
RUN USER=root cargo new --bin app
WORKDIR /app

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# this build step will cache your dependencies
RUN apt update && \
    apt install -y libssl-dev openssl pkg-config
RUN cargo build --release
RUN rm src/*.rs

# copy your source tree
COPY ./src ./src

# build for release
RUN rm ./target/release/deps/*
RUN cargo build --release

# our final base
FROM debian:bookworm-slim
WORKDIR /app

# image 'debian:bookworm-slim' needs ca-certificates package for TLS
RUN apt-get update && \
    apt install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*


# Copy the schedule.csv file into the same directory as the binary
COPY input /app/

# Create a userdata dir which can be mounted later
RUN mkdir /app/userdata

# copy the build artifact from the build stage,
COPY --from=builder /app/target/release/dailybible-rs .

# set the startup command to run your binary
CMD ["./dailybible-rs"]