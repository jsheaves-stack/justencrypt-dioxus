# Stage 1: Build the Rust/Dioxus application
FROM rust:1.94-slim AS builder

# Install necessary dependencies
RUN apt-get update && apt-get install -y \
    curl \
    pkg-config \
    libssl-dev \
    nodejs \
    npm \
    git \
    && rm -rf /var/lib/apt/lists/*

# Add the webassembly target
RUN rustup target add wasm32-unknown-unknown

# Install Dioxus CLI
RUN cargo install dioxus-cli --version 0.7.3

WORKDIR /app
COPY . .

# Install NPM dependencies (for tailwind) and build tailwind CSS
RUN npm install
RUN npm run build:css

# Pass the build version to the app
ARG APP_VERSION
ENV APP_VERSION=${APP_VERSION}

# Build the Dioxus app for release on the web platform
RUN dx build --release --platform web

# Stage 2: Serve the application with Caddy
FROM caddy:alpine

# Copy the built files from the builder
COPY --from=builder /app/target/dx/justencrypt-client/release/web/public /usr/share/caddy

# Copy Caddyfile
COPY Caddyfile /etc/caddy/Caddyfile

# Add the entrypoint script
COPY docker-entrypoint.sh /usr/local/bin/docker-entrypoint.sh
RUN chmod +x /usr/local/bin/docker-entrypoint.sh

EXPOSE 80

ENTRYPOINT ["/usr/local/bin/docker-entrypoint.sh"]
CMD ["caddy", "run", "--config", "/etc/caddy/Caddyfile", "--adapter", "caddyfile"]
