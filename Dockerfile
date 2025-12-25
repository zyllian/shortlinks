FROM rust:latest AS builder

WORKDIR /usr/src/shortlinks
COPY . .

RUN cargo install --path .

WORKDIR /shortlinks

CMD ["shortlinks"]

LABEL "org.opencontainers.image.source" "https://github.com/zyllian/shortlinks"
