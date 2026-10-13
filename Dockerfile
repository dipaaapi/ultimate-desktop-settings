# syntax=docker/dockerfile:1
#
# Ultimate Windows Desktop Settings
#
#   docker compose up -d                       -> design preview at http://localhost:8080
#   docker compose --profile windows run --rm windows-app
#                                              -> builds the Windows app into ./out
#
# A container cannot change the settings of the PC it runs on, so the preview
# uses sample data. The real app is the Windows .exe built by the second target.

ARG NODE_IMAGE=node:24-bookworm-slim
ARG RUST_IMAGE=rust:1.97-slim-bookworm
ARG NGINX_IMAGE=nginxinc/nginx-unprivileged:1.29-alpine

# ---- Interface (React + TypeScript) ----
FROM ${NODE_IMAGE} AS web
WORKDIR /app
COPY package.json package-lock.json ./
RUN --mount=type=cache,target=/root/.npm npm ci --no-audit --no-fund
COPY index.html vite.config.ts tsconfig.json tsconfig.app.json tsconfig.node.json ./
COPY public public
COPY src src
RUN npm run build

# ---- Design preview served on port 8080 ----
FROM ${NGINX_IMAGE} AS preview
LABEL org.opencontainers.image.title="Ultimate Windows Desktop Settings (preview)" \
      org.opencontainers.image.description="Browser preview of the silver rod and beads with sample data. Open http://localhost:8080" \
      org.opencontainers.image.authors="EdMaster28 <dipaaapi8@gmail.com>" \
      org.opencontainers.image.version="2.0.0"
COPY docker/nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=web /app/dist /usr/share/nginx/html
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s CMD wget -q -O /dev/null http://127.0.0.1:8080/ || exit 1

# ---- Windows app (Tauri + Rust), cross-compiled with cargo-xwin ----
FROM ${RUST_IMAGE} AS windows-build
RUN apt-get update \
 && apt-get install -y --no-install-recommends clang lld llvm \
 && rm -rf /var/lib/apt/lists/*
RUN rustup target add x86_64-pc-windows-msvc \
 && cargo install --locked cargo-xwin@0.23.1
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY crates crates
COPY src-tauri src-tauri
COPY --from=web /app/dist dist
# The first build downloads the Microsoft C runtime headers (accepting their licence) and caches them.
ENV XWIN_ACCEPT_LICENSE=1
RUN --mount=type=cache,target=/root/.cache/cargo-xwin \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    cargo xwin build --release --locked \
      -p ultimate-windows-desktop-settings \
      --target x86_64-pc-windows-msvc \
      --features tauri/custom-protocol \
 && mkdir -p /out \
 && cp target/x86_64-pc-windows-msvc/release/ultimate-windows-desktop-settings.exe /out/

# `docker build --target export --output out .` writes the .exe straight to ./out
FROM scratch AS export
COPY --from=windows-build /out/ /

# `docker compose --profile windows run --rm windows-app` copies the .exe to ./out
FROM ${NODE_IMAGE} AS windows-app
COPY --from=windows-build /out/ /app-out/
CMD ["sh", "-c", "cp -v /app-out/*.exe /export/ && echo 'Done. Run out\\\\ultimate-windows-desktop-settings.exe on Windows.'"]
