FROM rust:1.70-bookworm AS cli-build

RUN apt-get update \
    && apt-get install -y --no-install-recommends build-essential \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY .cargo .cargo
COPY vendor vendor
COPY crates crates
RUN cargo build --release --locked --offline -p vikas-cli

FROM node:20-bookworm-slim AS workbench-build

WORKDIR /app
COPY package.json package-lock.json ./
RUN npm ci --no-audit --no-fund
COPY index.html server.ts tsconfig.json vite.config.ts ./
COPY src src
COPY examples examples
COPY vikas.toml ./
RUN npm run build

FROM node:20-bookworm-slim AS runtime

ENV NODE_ENV=production
LABEL org.opencontainers.image.source="https://github.com/ginnela-sairam/Vikas"
LABEL org.opencontainers.image.description="VIKAS RTL simulation, static latch/CDC signoff, and binary waveform workbench"
LABEL org.opencontainers.image.licenses="MIT OR Apache-2.0"
WORKDIR /work
COPY --from=cli-build /src/target/release/vikas /usr/local/bin/vikas
COPY --from=workbench-build /app /app
RUN mkdir -p /work && chown node:node /work

USER node
EXPOSE 3000
ENTRYPOINT ["/usr/local/bin/vikas"]
CMD ["version"]
