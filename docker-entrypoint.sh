#!/bin/sh
set -e

# Generate env.js containing the environment variable
cat <<EOF > /usr/share/nginx/html/env.js
window.API_URL = "${API_URL:-http://localhost:8000}";
EOF

# Inject the script tag into index.html before the closing </head> tag, if not present
if ! grep -q '<script src="/env.js"></script>' /usr/share/nginx/html/index.html; then
  sed -i 's|</head>|<script src="/env.js"></script></head>|' /usr/share/nginx/html/index.html
fi

exit 0
