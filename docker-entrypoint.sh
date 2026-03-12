#!/bin/sh
set -e

# Generate env.js containing the environment variable
# The application uses this to know where the API is located.
ROOT_DIR="/usr/share/caddy"

cat <<EOF > "$ROOT_DIR/env.js"
window.API_URL = "${API_URL:-http://localhost:8000}";
EOF

# Inject the script tag into index.html before the closing </head> tag, if not present
if [ -f "$ROOT_DIR/index.html" ]; then
    if ! grep -q '<script src="/env.js"></script>' "$ROOT_DIR/index.html"; then
      sed -i 's|</head>|<script src="/env.js"></script></head>|' "$ROOT_DIR/index.html"
    fi
fi

# Execute the passed command (e.g., caddy)
exec "$@"
