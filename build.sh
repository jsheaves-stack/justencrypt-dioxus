#!/bin/bash

# Build the app for one or more platforms/profiles.
#
# Desktop builds are bundled as an AppImage (via `dx bundle`), since that's
# the shippable desktop artifact. Web builds use `dx build` since there's
# nothing to bundle - it just produces the static site.
#
# Usage: ./build.sh [web|desktop|all] [debug|release|all]
#
# Examples:
#   ./build.sh                  # desktop AppImage, debug + release
#   ./build.sh desktop release  # just the desktop release AppImage
#   ./build.sh web              # web debug + release
#   ./build.sh all              # web + desktop, debug + release

cd "$(dirname "${BASH_SOURCE[0]}")" || exit 1

# cargo-installed tools (like dx) live here; this script runs under bash
# regardless of your login shell, so make sure it's on PATH.
export PATH="$HOME/.cargo/bin:$PATH"

usage() {
    echo "Usage: $0 [web|desktop|all] [debug|release|all]"
    exit 1
}

PLATFORM="${1:-desktop}"
PROFILE="${2:-all}"

case "$PLATFORM" in
    web|desktop|all) ;;
    *) echo "Error: unknown platform '$PLATFORM'"; usage ;;
esac

case "$PROFILE" in
    debug|release|all) ;;
    *) echo "Error: unknown profile '$PROFILE'"; usage ;;
esac

if ! command -v dx >/dev/null 2>&1; then
    echo "Error: dx (Dioxus CLI) not found on PATH"
    exit 1
fi

if [ "$PLATFORM" = "all" ]; then
    PLATFORMS=(web desktop)
else
    PLATFORMS=("$PLATFORM")
fi

if [ "$PROFILE" = "all" ]; then
    PROFILES=(debug release)
else
    PROFILES=("$PROFILE")
fi

echo "Building Tailwind CSS..."
npm run build:css
if [ $? -ne 0 ]; then
    echo "Error: Tailwind build failed"
    exit 1
fi

for plat in "${PLATFORMS[@]}"; do
    for prof in "${PROFILES[@]}"; do
        echo ""

        if [ "$plat" = "desktop" ]; then
            echo "==> Bundling desktop AppImage ($prof)"

            ARGS=(--desktop --package-types appimage)
            if [ "$prof" = "release" ]; then
                ARGS+=(--release)
            fi

            NO_STRIP=true dx bundle "${ARGS[@]}"
            if [ $? -ne 0 ]; then
                echo "Error: desktop bundle failed ($prof)"
                exit 1
            fi
        else
            echo "==> Building $plat ($prof)"

            ARGS=(--platform "$plat")
            if [ "$prof" = "release" ]; then
                ARGS+=(--release)
            fi

            dx build "${ARGS[@]}"
            if [ $? -ne 0 ]; then
                echo "Error: build failed for $plat ($prof)"
                exit 1
            fi
        fi

        echo "==> Done: $plat ($prof)"
    done
done

echo ""
echo "All builds complete."
