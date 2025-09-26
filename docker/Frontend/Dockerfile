FROM hugomods/hugo:latest AS builder

# Build arguments (can be provided via docker compose build args)
ARG BASE_URL="/"

WORKDIR /src
# Copy site sources from repo root (build context should be repo root)
COPY frontend/ /src

# Build static site
ENV HUGO_ENV=production
RUN hugo --gc --minify --baseURL "${BASE_URL}"

FROM nginx:alpine AS runtime

# Copy built site into nginx html root
COPY --from=builder /src/public /usr/share/nginx/html

# Configure nginx to listen on 1313 to match existing reverse proxy
RUN printf '%s\n' \
    'server {' \
    '  listen 1313;' \
    '  server_name _;' \
    '  root /usr/share/nginx/html;' \
    '  index index.html;' \
    '  location / { try_files $uri $uri/ =404; }' \
    '}' > /etc/nginx/conf.d/default.conf

EXPOSE 1313
CMD ["nginx", "-g", "daemon off;"]
