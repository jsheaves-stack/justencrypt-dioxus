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

# Build the Dioxus app for release on the web platform
RUN dx build --release --platform web

# Stage 2: Serve the application with Nginx
FROM nginx:alpine

# Configure nginx to route all traffic to index.html for client-side routing
# We'll inline a simple configuration for SPAs.
RUN echo 'server { \
    listen 80; \
    server_name localhost; \
    root /usr/share/nginx/html; \
    index index.html; \
    location / { \
    try_files $uri $uri/ /index.html; \
    } \
    }' > /etc/nginx/conf.d/default.conf

# Copy the built files from the builder
COPY --from=builder /app/target/dx/justencrypt-client/release/web/public /usr/share/nginx/html

# Add the script to inject the runtime API_URL environment variable
COPY docker-entrypoint.sh /docker-entrypoint.d/99-env.sh
RUN chmod +x /docker-entrypoint.d/99-env.sh

EXPOSE 80
