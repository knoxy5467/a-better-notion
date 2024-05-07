FROM ubuntu:latest

COPY ./target/debug/server /usr/local/bin/server
COPY ./docker_settings.toml /usr/local/bin/Server.toml
COPY ./docker_settings.toml ./Server.toml
EXPOSE 8080

# Set the startup command to run your binary
CMD ["server"]