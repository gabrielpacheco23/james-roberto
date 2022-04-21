FROM rust:1.60 as builder
WORKDIR /james-roberto
COPY . .
RUN cargo install --path .

FROM ubuntu:20.04
RUN apt-get update && apt-get install libopus-dev libopus0 opus-tools
COPY --from=builder /usr/local/cargo/bin/james-roberto /usr/local/bin/james-roberto
CMD ["james-roberto"]

