# Stage 1: Build the Angular application
FROM node:latest as build

WORKDIR /app

# Copy package.json and package-lock.json
COPY package*.json ./

# Install dependencies
RUN npm install

# Copy the rest of the application code
COPY . .

# Build the application with production configuration
RUN npm run build -- --configuration=production

# Stage 2: Serve the application with Nginx
FROM nginx:alpine

# Copy the build output to replace the default nginx contents
COPY --from=build /app/dist/oss-frontend /usr/share/nginx/html

# Copy custom nginx configuration
COPY ./nginx.conf /etc/nginx/nginx.conf

# When the container starts, replace the env.js with values from environment variables
COPY ./env.sh /usr/share/nginx/html/
RUN chmod +x /usr/share/nginx/html/env.sh

# Create env.js file
RUN echo "window.env = {};" > /usr/share/nginx/html/env.js

# Start Nginx server and run the env.sh script
CMD ["/bin/sh", "-c", "/usr/share/nginx/html/env.sh && nginx -g \"daemon off;\""]

# Expose port 80
EXPOSE 80
