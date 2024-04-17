# Use an official Rust runtime as a parent image
FROM rust:1.54 as builder

# Set the working directory in the image to /usr/src/server
WORKDIR /usr/src/server

# Copy the server directory contents into the image at /usr/src/server
COPY server .

# Build the application
RUN cargo build --release

# Start a new stage. This is for creating the final image
FROM debian:buster-slim

# Copy the binary from the builder stage to the final stage
COPY --from=builder /usr/src/server/target/release/server /usr/local/bin/server

# Set the startup command to run your binary
CMD ["server"]