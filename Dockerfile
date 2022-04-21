FROM rust:1.60 as builder
WORKDIR /james-roberto
COPY . .
RUN cargo install --path .

FROM gliderlabs/alpine:3.3
RUN apk add --update-cache libopus-dev
COPY --from=builder /usr/local/cargo/bin/james-roberto /usr/local/bin/james-roberto
CMD ["james-roberto"]

