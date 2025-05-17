#!/bin/sh

# Replace environment variables in the env.js file
echo "window.env = {" > /usr/share/nginx/html/env.js
echo "  apiUrl: '${API_URL:-http://localhost:3000/api}'," >> /usr/share/nginx/html/env.js
echo "};" >> /usr/share/nginx/html/env.js

# Find and replace environment variables in the main.*.js file
# This is needed because Angular's environment.prod.ts has placeholders like ${API_URL}
find /usr/share/nginx/html -type f -name "main.*.js" -exec sed -i "s|\${API_URL}|${API_URL:-http://localhost:3000/api}|g" {} \;

echo "Environment variables injected."