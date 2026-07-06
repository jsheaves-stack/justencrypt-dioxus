# JustEncrypt Client

A cross-platform file encryption service client application built using Rust and Dioxus.

## Features

* **Cross-Platform Support**: Runs as a Web application (WASM) in the browser or natively on Desktop.
* **File Explorer**: Navigate through your encrypted files and folders.
  * **Sorting**: Organizes items alphabetically by Name or strictly by file Type.
  * **Mobile Responsive**: Dynamically scales grids and action bars to beautifully fit on all devices.
* **Authentication**: Login securely to access your encrypted vault.
* **Hidden Files**: Toggle visibility of hidden files and folders by pressing the `.` (period) key.
* **Modern UI**: Styled with Tailwind CSS, featuring a built-in Dark Mode.

## Tech Stack

* **Framework**: [Rust](https://www.rust-lang.org/) with [Dioxus](https://dioxuslabs.com/) `v0.7.3`
* **Styling**: [Tailwind CSS](https://tailwindcss.com/)
* **HTTP Client**: `reqwest` (Handles API communication, streams, and cookies across target architectures)
* **Backend**: Expects a JustEncrypt Backend API running at `http://localhost:8000` by default (Configurable via `API_URL` environment variable).

## Prerequisites

Before building or developing, ensure you have the following installed:

1. [Rust & Cargo](https://rustup.rs/) (v1.94+ recommended)
2. [Node.js & npm](https://docs.npmjs.com/downloading-and-installing-node-js-and-npm) (for Tailwind CSS)
3. [Dioxus CLI](https://dioxuslabs.com/learn/0.6/getting_started):
   ```bash
   cargo install --locked dioxus-cli --version 0.7.3
   ```
4. WebAssembly target (if building for web):
   ```bash
   rustup target add wasm32-unknown-unknown
   ```

## Development

### 1. Install Dependencies

First, install the required Node.js dependencies for Tailwind CSS:
```bash
npm install
```

### 2. Run the Tailwind CSS Compiler

You need to compile the Tailwind CSS so the application has the correct styling. Run the following command in a terminal and leave it running in the background:

```bash
npx tailwindcss -i ./input.css -o ./assets/tailwind.css --watch
```
*(Alternatively, run `npm run build:css` for a one-time compilation).*

### 3. Start the Dioxus Dev Server

In a new terminal window, start the Dioxus development server for the web platform:

```bash
dx serve --platform web --hot-reload true
```

* Open your browser to http://localhost:8080.
* Ensure your backend API is running at `http://localhost:8000`.

## Building for Production

### Web (WASM)

1. Build the CSS for production:
   ```bash
   npm run build:css
   ```
2. Build the web app for release:
   ```bash
   dx build --release --platform web
   ```
   The built artifacts will be located at `target/dx/justencrypt-client/release/web/public`. These files can be served by any static web server (like Nginx, Apache, or Caddy).

### Desktop (Native)

1. Build the CSS for production:
   ```bash
   npm run build:css
   ```
2. Build the desktop app for release:
   ```bash
   dx build --release --platform desktop
   ```

## Docker Deployment

The application includes a `Dockerfile` that builds the web application and serves it using a lightweight Caddy container. It supports injecting the backend API URL at runtime.

### 1. Build the Docker Image

```bash
docker build -t justencrypt-client .
```

### 2. Run the Container

You can run the container and specify the backend API URL using the `API_URL` environment variable. If omitted, it defaults to `/api` or the fallback hardcoded in the app.

```bash
docker run -p 8080:80 -e API_URL="https://api.yourdomain.com" justencrypt-client
```

* The app will now be accessible at http://localhost:8080.

## Project Architecture Notes

The application uses conditional compilation to support both web and desktop environments:
- **`#[cfg(target_arch = "wasm32")]`**: Code specific to the Web platform (e.g., using `js-sys`, handling fetch API credentials).
- **`#[cfg(not(target_arch = "wasm32"))]`**: Code specific to the Desktop native application (e.g., handling cookies with `reqwest::cookie::Jar`).

When contributing or adding new dependencies, ensure they are compatible with both WebAssembly and Native Desktop targets, or use conditional compilation attributes to isolate platform-specific code.