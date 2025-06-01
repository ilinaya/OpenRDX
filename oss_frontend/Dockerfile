FROM node:24.0.1-alpine3.21 AS builder
WORKDIR /app

# Only copy package files first for better caching
COPY package.json package-lock.json ./
RUN npm ci

# Now copy the rest of the source
COPY . .

ARG COMMIT_SHA
ENV COMMIT_SHA=$COMMIT_SHA

ARG BUILD_TIMESTAMP
ENV BUILD_TIMESTAMP=$BUILD_TIMESTAMP

ARG API_URL
ENV API_URL=$API_URL

# Debug output
RUN echo "Building with API_URL: $API_URL"


RUN npm run build:production


FROM nginx:1.27.5-alpine-slim

# Copy nginx config and built app
COPY nginx.conf /etc/nginx/nginx.conf
COPY --chown=nginx:nginx --from=builder /app/dist /usr/share/nginx/html

EXPOSE 80
ENTRYPOINT ["nginx", "-g", "daemon off;"]
