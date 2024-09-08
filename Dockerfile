FROM rust:1.81-alpine as builder
RUN apk add --no-cache musl-dev
WORKDIR /usr/src/alien-network-discord-bot
COPY . .
RUN cargo install --locked --path .

FROM alpine:3.20
COPY --from=builder /usr/local/cargo/bin/alien-network-discord-bot /usr/local/bin/alien-network-discord-bot
CMD ["alien-network-discord-bot"]