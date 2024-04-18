FROM ubuntu:latest

COPY ./server /usr/local/bin/server
COPY ./Server.toml /usr/local/bin/Server.toml
COPY ./Server.toml ./Server.toml
EXPOSE 8080

# Set the startup command to run your binary
CMD ["server"]