#!/bin/bash

# Configuration
REPO="onedev.sheavesnet.ca/justencrypt-dioxus/justencrypt-dioxus"
CARGO_FILE="Cargo.toml"

# Get version from Cargo.toml
VERSION=$(grep '^version =' "$CARGO_FILE" | head -n 1 | cut -d '"' -f 2)
if [ -z "$VERSION" ]; then
    echo "Error: Could not extract version from $CARGO_FILE"
    exit 1
fi

# Get current date (YYYYMMDD)
DATE=$(date +%Y%m%d)

# Find existing local tags for today and get the max iteration
# This matches tags like VERSION-DATE.ITERATION (e.g., 0.1.0-20260318.1)
LAST_ITERATION=$(docker images --format "{{.Tag}}" "$REPO" | \
    grep -E "^${VERSION}-${DATE}\.[0-9]+$" | \
    sed -E "s/.*\.([0-9]+)$/\1/" | \
    sort -rn | head -n 1)

# Default to iteration 1, or increment last iteration
if [ -z "$LAST_ITERATION" ]; then
    ITERATION=1
else
    ITERATION=$((LAST_ITERATION + 1))
fi

FULL_TAG="${VERSION}-${DATE}.${ITERATION}"
FULL_IMAGE="${REPO}:${FULL_TAG}"
LATEST_IMAGE="${REPO}:latest"

echo "Building version: $FULL_TAG (Iteration: $ITERATION)"

echo "Building web image..."
# Build the web image
docker build \
    --target web \
    --build-arg APP_VERSION="$FULL_TAG" \
    --build-arg BUILD_NUMBER="$ITERATION" \
    -t "$FULL_IMAGE" \
    -t "$LATEST_IMAGE" .

# Check build success
if [ $? -ne 0 ]; then
    echo "Error: Docker web build failed"
    exit 1
fi

echo "Building desktop AppImage..."
# Build the desktop AppImage
docker build \
    --target desktop-artifacts \
    --build-arg APP_VERSION="$FULL_TAG" \
    --build-arg BUILD_NUMBER="$ITERATION" \
    --output type=local,dest=./dist .

# Check build success
if [ $? -ne 0 ]; then
    echo "Error: Docker desktop build failed"
    exit 1
fi

# Push the images
echo "Pushing images..."
docker push "$FULL_IMAGE"
docker push "$LATEST_IMAGE"

echo "Successfully built and pushed $FULL_TAG and latest"
echo "Desktop AppImage has been extracted to ./dist"
