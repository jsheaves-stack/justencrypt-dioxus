# JustEncrypt Client - Project Context

This file serves as the primary technical context and architectural reference for the `justencrypt-client` project. It provides AI agents and new developers with essential insights to work effectively and safely in the codebase.

## Project Overview
- **Name**: JustEncrypt Client
- **Description**: A cross-platform file encryption service client application.
- **Tech Stack**:
  - **Framework**: Rust with [Dioxus](https://dioxuslabs.com/) `v0.7.3` (Targets both `web` via WASM and `desktop` natively).
  - **Styling**: Tailwind CSS (compiled via Node/npm), configured with custom utility wrappers for Dark Mode.
  - **HTTP Client**: `reqwest` (handling API communication, streams, and cookies across target architectures).
- **Backend API**: The client expects a backend running at `http://localhost:8000` (defined in `API_URL` in `src/main.rs`).

## Target Architectures
The application is strictly designed to build for two targets using conditional compilation (`#[cfg(target_arch = "wasm32")]` and `#[cfg(not(target_arch = "wasm32"))]`):
- **Web (WASM)**: Uses browser APIs, handles cookies via `.fetch_credentials_include()`, built using `js-sys` and `wasm-streams`.
- **Desktop**: Native Rust binary utilizing `dioxus-desktop` (built on Tao/Wry), handling cookies using `reqwest::cookie::Jar`.
**Important**: Any new library dependency, system API call, windowing change, or file I/O must respect these target compilation flags.

## Directory Structure
- `src/main.rs`: The application entry point, setting up the router, HTTP client, logging, and global states.
- `src/components/`: Contains all Dioxus UI elements.
  - `src/components/ui/`: Reusable, purely presentational UI primitives (e.g., `button.rs`, `card.rs`, `header.rs`, `image.rs`, `input.rs`, `label.rs`).
  - Feature components: `file_explorer`, `home`, `login_form`, `file`, `folder`, `create_folder_modal`.
- `src/requests/`: Handles network interactions with the backend API.
  - `file.rs`, `folder.rs`: Interactions related to data (fetching, uploading).
  - `session.rs`: Handles authentication.
  - `mod.rs`: Provides the `RequestBuilderExt` trait to manage cookie passing uniformly (`with_credentials_if_wasm()`).
- `assets/`: Static resources like icons, vectors (`header.svg`), and compiled CSS (`tailwind.css`, `main.css`).
- `input.css`: The source Tailwind file.

## Key Application Mechanics & State
- **Global Context**: Managed via Dioxus `use_context_provider::<Signal<T>>()`.
  - `AppState`: Struct containing `_theme` (default: "light"), `authenticated` (bool), `create_folder_modal` (bool), and `needs_refetch` (bool).
  - `ReqwestClient`: A `Clone`able wrapper around `reqwest::Client` customized per target architecture.
- **Routing**: `dioxus::prelude::Router` is used.
  - `/` -> `Home` (redirects to `/login` or `/folder/` based on auth state).
  - `/login` -> `Login` (renders login form).
  - `/folder/:..path` -> `FileExplorer` (renders directory contents). Note that the route uses a catch-all parameter (`:..path`) resolved as `Vec<String>`.
- **Data Fetching Pattern**:
  - API calls rely on `use_resource`, passing in context such as `reqwest_client_context` and path via `use_reactive!`.
  - A successful mutation (e.g., file upload, folder creation) toggles `app_state.write().needs_refetch = true;`. An effect hook (`use_effect`) listens to this and triggers `resource.restart()` to refresh data.
- **Image Handling**: The `Image` component (in `src/components/ui/image.rs`) manually fetches image data over the network via `reqwest` and renders them as base64-encoded strings `src="data:{content_type};base64,{b64}"`.

## Hidden Files Feature (Gemini Memory)
- As noted in previous interactions, the file explorer is intended to support hiding/showing hidden files and folders, which is toggled via the `.` key. Agents should preserve this behavior if found, and account for it when modifying key events or rendering logic.

## Development Commands
1. **Styling (Tailwind CSS)**:
   - Make sure to compile CSS before or during app execution.
   - Run watcher: `npx tailwindcss -i ./input.css -o ./assets/tailwind.css --watch`
   - Or use the npm script: `npm run build:css`
2. **Running the App (Dioxus CLI)**:
   - Web Platform (default dev server): `dx serve --platform web --hot-reload true` (runs on `http://localhost:8080`).

## Coding Conventions & Guidelines
- **Dioxus Macros**: UI is built utilizing the `rsx!` macro.
- **Styling Strategy**: Utility-first styling with Tailwind CSS embedded directly inside the `rsx!` markup class attributes. The `dark` class is globally applied on the main wrapping div.
- **State Mutation**: Due to Dioxus's `Signal` implementation, state mutation involves `.read()` and `.write()` semantics on contexts and hooks.
- **Error Handling**: API logic often maps responses and maps/bubbles up errors cleanly (e.g., mapping `reqwest` errors to unit errors `()` internally in basic components).
