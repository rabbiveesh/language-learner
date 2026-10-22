# Fala: the web app built with Node, the server built with Rust, run under Litestream.

FROM node:24-bookworm-slim AS web
WORKDIR /app/web
COPY web/package.json web/package-lock.json ./
RUN npm ci
COPY web/ ./
RUN npm run build-only

FROM rust:1-bookworm AS server
WORKDIR /app
COPY . .
RUN cargo build --release --locked -p server

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 sqlite3 \
    && rm -rf /var/lib/apt/lists/*
ARG LITESTREAM_VERSION=0.3.13
ADD https://github.com/benbjohnson/litestream/releases/download/v${LITESTREAM_VERSION}/litestream-v${LITESTREAM_VERSION}-linux-amd64.deb /tmp/litestream.deb
RUN dpkg -i /tmp/litestream.deb && rm /tmp/litestream.deb
COPY --from=server /app/target/release/server /usr/local/bin/fala
COPY --from=web /app/web/dist /app/web/dist
COPY deploy/litestream.yml /etc/litestream.yml
COPY deploy/start.sh /usr/local/bin/start
ENV WEB_DIST=/app/web/dist \
    DATABASE_URL=sqlite:///data/fala.db \
    PORT=8080
EXPOSE 8080
CMD ["start"]
