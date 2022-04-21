FROM rust:1.60 as builder
WORKDIR /james-roberto
COPY . .
RUN cargo install --path .

FROM debian:buster-slim
RUN apt-get update && rm -rf /var/lib/apt/lists/*
RUN apt-get install libopus0 libopus-dev opus-tools
COPY --from=builder /usr/local/cargo/bin/james-roberto /usr/local/bin/james-roberto
CMD ["james-roberto"]

