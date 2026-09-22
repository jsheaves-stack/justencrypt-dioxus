# syntax=docker/dockerfile:1

# Requires BuildKit/buildx - this relies on cache mounts (`RUN --mount=type=cache`)
# and is not compatible with the legacy (non-BuildKit) builder.

# Toolchain setup shared by both build stages below: no project source is
# copied in yet, so editing source files never invalidates this (or the
# desktop-only apt-get further down, which is also placed before its stage's
# COPY for the same reason) - only actual toolchain/dependency changes do.
FROM rust:1.94-slim AS toolchain

RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt/lists,sharing=locked \
    rm -f /etc/apt/apt.conf.d/docker-clean && \
    apt-get update && apt-get install -y \
    curl \
    pkg-config \
    libssl-dev \
    nodejs \
    npm \
    git

# Add the webassembly target
RUN rustup target add wasm32-unknown-unknown

# Install Dioxus CLI
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cargo install --locked dioxus-cli --version 0.7.3

WORKDIR /app

# ---------------------------------------------------------------------------
# Web target: build the wasm app, then serve it with Caddy.
# This is the default target (`docker build .`), unchanged from before.
#
# `target/` is cache-mounted for faster incremental rebuilds, so the actual
# build output has to be copied out to a normal (non-cache-mounted) path
# before the RUN instruction ends, or it would vanish along with the mount.
# ---------------------------------------------------------------------------
FROM toolchain AS web-builder

COPY . .

# Install NPM dependencies (for tailwind) and build tailwind CSS
RUN --mount=type=cache,target=/root/.npm \
    npm install
RUN npm run build:css

# Pass the build version and number to the app
ARG APP_VERSION
ARG BUILD_NUMBER=1

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target,id=cargo-target-web \
    if [ -z "$APP_VERSION" ]; then \
        VERSION=$(grep '^version =' Cargo.toml | head -n 1 | cut -d '"' -f 2); \
        DATE=$(date +%Y%m%d); \
        export APP_VERSION="${VERSION}-${DATE}.${BUILD_NUMBER}"; \
    else \
        export APP_VERSION="${APP_VERSION}"; \
    fi && \
    export BUILD_NUMBER="${BUILD_NUMBER}" && \
    dx build --release --platform web && \
    mkdir -p /output && \
    cp -r target/dx/justencrypt-client/release/web/public /output/public

# ---------------------------------------------------------------------------
# Desktop toolchain: deliberately NOT based on the Debian image used above.
#
# `dx bundle`'s AppImage packaging embeds its own copies of GTK/WebKitGTK/
# graphics-stack shared libraries, taken from whatever OS built it. Debian
# trixie's GTK3 (3.24.49) hits a GTK-level EGL-display-init bug
# (`eglGetPlatformDisplay` -> EGL_BAD_PARAMETER, aborting before WebKit even
# picks a render backend) when the AppImage's own AppRun script forces
# GDK_BACKEND=x11 and the app ends up running through XWayland on some GPUs -
# verified by bisecting against a native build on this GPU, which uses GTK3
# 3.24.52 and doesn't hit it. Fedora tracks current GTK3/WebKitGTK closely
# enough to sidestep this (verified 3.24.52 too) while still being a real,
# tested release rather than a rolling one - see the diagnosis in the commit
# this comment shipped with for the full bisection.
# ---------------------------------------------------------------------------
FROM fedora:44 AS desktop-toolchain

RUN --mount=type=cache,target=/var/cache/dnf,sharing=locked \
    dnf install -y \
    gtk3-devel \
    webkit2gtk4.1-devel \
    libsoup3-devel \
    libxdo-devel \
    librsvg2-devel \
    gpgme \
    file \
    desktop-file-utils \
    curl \
    git \
    gcc \
    make \
    pkgconf-pkg-config \
    openssl-devel \
    nodejs22 \
    nodejs22-npm

# Pin the same rustc version the Debian-based toolchain stage above uses,
# via rustup rather than a distro package, so both stages build with the
# same compiler regardless of what Fedora currently ships.
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y --profile minimal --default-toolchain 1.94.0
ENV PATH="/root/.cargo/bin:${PATH}"

# Install Dioxus CLI
RUN --mount=type=cache,target=/root/.cargo/registry \
    --mount=type=cache,target=/root/.cargo/git \
    cargo install --locked dioxus-cli --version 0.7.3

WORKDIR /app

# ---------------------------------------------------------------------------
# Desktop target: build the native Linux app and bundle it as an AppImage.
#
#   docker build --target desktop-artifacts --output type=local,dest=./dist .
#
# copies the resulting .AppImage straight to ./dist on the host.
#
# NO_STRIP works around a broken `strip` bundled in linuxdeploy that can't
# handle the .relr.dyn sections modern toolchains emit. The tauri cache
# mount avoids re-downloading linuxdeploy/appimagetool on every build; unlike
# the web stage, `dist/` lives outside `/app/target` so it survives the RUN
# instruction without needing to be copied out separately.
# ---------------------------------------------------------------------------
FROM desktop-toolchain AS desktop-builder

COPY . .

RUN --mount=type=cache,target=/root/.npm \
    npm install
RUN npm run build:css

ARG APP_VERSION
ARG BUILD_NUMBER=1

ENV NO_STRIP=true
RUN --mount=type=cache,target=/root/.cargo/registry \
    --mount=type=cache,target=/root/.cargo/git \
    --mount=type=cache,target=/app/target,id=cargo-target-desktop \
    --mount=type=cache,target=/root/.cache/tauri \
    if [ -z "$APP_VERSION" ]; then \
        VERSION=$(grep '^version =' Cargo.toml | head -n 1 | cut -d '"' -f 2); \
        DATE=$(date +%Y%m%d); \
        export APP_VERSION="${VERSION}-${DATE}.${BUILD_NUMBER}"; \
    else \
        export APP_VERSION="${APP_VERSION}"; \
    fi && \
    export BUILD_NUMBER="${BUILD_NUMBER}" && \
    dx bundle --desktop --package-types appimage --release

FROM scratch AS desktop-artifacts
COPY --from=desktop-builder /app/dist/*.AppImage /

# ---------------------------------------------------------------------------
# Web target (continued): the actual runtime image. Kept as the last stage
# so it remains the default `docker build .` target.
# ---------------------------------------------------------------------------
FROM caddy:alpine AS web

# Copy the built files from the builder
COPY --from=web-builder /output/public /usr/share/caddy

# Copy Caddyfile
COPY Caddyfile /etc/caddy/Caddyfile

# Add the entrypoint script
COPY docker-entrypoint.sh /usr/local/bin/docker-entrypoint.sh
RUN chmod +x /usr/local/bin/docker-entrypoint.sh

EXPOSE 80

ENTRYPOINT ["/usr/local/bin/docker-entrypoint.sh"]
CMD ["caddy", "run", "--config", "/etc/caddy/Caddyfile", "--adapter", "caddyfile"]
