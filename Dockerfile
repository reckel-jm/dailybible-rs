# Build stage
FROM rust:1.80 AS builder

# Install musl-tools for static linking
RUN apt-get update && apt-get install -y musl-tools

# Add the musl target for compatibility with Alpine
RUN rustup target add x86_64-unknown-linux-musl

# Set the working directory
WORKDIR /usr/src/dailybible-rs

# Copy all project files
COPY . .

# Build the project with the musl target
RUN cargo build --release --target x86_64-unknown-linux-musl

# Runtime stage
FROM alpine:3.18

# Install CA certificates for HTTPS requests
RUN apk add --no-cache ca-certificates

# Create a non-root user for security
RUN adduser -D dailybible

# Set the working directory for the runtime
WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/dailybible-rs/target/x86_64-unknown-linux-musl/release/dailybible-rs /app/

# Copy the schedule.csv file into the same directory as the binary
COPY schedule.csv /app/

# Set the user to run the container
USER dailybible

# Run the bot
CMD ["/app/dailybible-rs"]
