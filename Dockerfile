FROM rust:1.60 as builder
WORKDIR /james-roberto
COPY . .
RUN cargo install --path .

FROM ubuntu:20.04
RUN apt-get update && apt-get install -y libopus-dev libopus0 opus-tools && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/james-roberto /usr/local/bin/james-roberto
CMD ["james-roberto"]

