#!/bin/sh
# Works out this build's version, <Cargo.toml version>-<UTC date>.<n>, where
# n is 1 for the first image pushed that day and counts up from there. The
# count comes from the tags already in the container registry.
#
# Writes into build-info/:
#   version     the version, e.g. 0.1.0-20261014.2
#   image-tags  the image tags to push, separated by spaces: the version,
#               plus latest when building main
#
# Reads from the environment:
#   REGISTRY_URL       registry base URL, e.g. https://onedev.example.com
#   IMAGE              full image name without a tag, e.g. onedev.example.com/team/app
#   REGISTRY_USER      registry login name
#   REGISTRY_PASSWORD  registry password or access token
#   BRANCH             the branch being built

set -eu

: "${REGISTRY_URL:?}" "${IMAGE:?}" "${REGISTRY_USER:?}" "${REGISTRY_PASSWORD:?}"

cargo_version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)

if [ -z "$cargo_version" ]; then
    echo "No version found in Cargo.toml." >&2
    exit 1
fi

prefix="$cargo_version-$(date -u +%Y%m%d)."

# The repository part of the image name, e.g. team/app.
repository=${IMAGE#*/}

token=$(curl -fsS -u "$REGISTRY_USER:$REGISTRY_PASSWORD" \
    "$REGISTRY_URL/v2/token?service=onedev&scope=repository:$repository:pull" |
    sed -n 's/.*"token" *: *"\([^"]*\)".*/\1/p')

if [ -z "$token" ]; then
    echo "The registry didn't return a token." >&2
    exit 1
fi

status=$(curl -sS -o tags.json -w '%{http_code}' \
    -H "Authorization: Bearer $token" \
    "$REGISTRY_URL/v2/$repository/tags/list")

case "$status" in
    200) ;;
    # The image hasn't been pushed yet.
    404) echo '{}' > tags.json ;;
    *)
        echo "Listing the image's tags failed with HTTP $status:" >&2
        cat tags.json >&2
        exit 1
        ;;
esac

last=$(grep -o '"[^"]*"' tags.json | tr -d '"' |
    awk -v prefix="$prefix" '
        index($0, prefix) == 1 {
            n = substr($0, length(prefix) + 1)
            if (n ~ /^[0-9]+$/ && n + 0 > max) max = n + 0
        }
        END { print max + 0 }')

rm -f tags.json

version="$prefix$((last + 1))"
tags="$IMAGE:$version"

if [ "${BRANCH:-}" = main ]; then
    tags="$tags $IMAGE:latest"
fi

mkdir -p build-info
printf '%s' "$version" > build-info/version
printf '%s' "$tags" > build-info/image-tags

echo "Version: $version"
echo "Image tags: $tags"
