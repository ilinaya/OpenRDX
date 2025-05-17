# OSS Frontend

This is an Angular-based frontend application for the OSS (Operations Support System) project.

## Features

- JWT Authentication
- User Management
- Device Management
- Admin Settings

## Development

### Prerequisites

- Node.js (v16 or later)
- npm (v7 or later)

### Setup

1. Clone the repository
2. Install dependencies:
   ```
   npm install
   ```
3. Start the development server:
   ```
   npm start
   ```
4. Open your browser and navigate to `http://localhost:4200`

## Docker Deployment

### Option 1: Using Docker Directly

#### Building the Docker Image

```bash
docker build -t oss-frontend .
```

#### Running the Docker Container

```bash
docker run -d -p 80:80 -e API_URL=http://your-api-url/api oss-frontend
```

#### Environment Variables

The following environment variables can be passed to the Docker container:

- `API_URL`: The URL of the backend API (default: `http://localhost:3000/api`)

Example:

```bash
docker run -d -p 80:80 \
  -e API_URL=http://api.example.com/api \
  oss-frontend
```

### Option 2: Using Docker Compose

For easier deployment, especially when running both frontend and backend together, you can use Docker Compose:

```bash
docker-compose up -d
```

This will start both the frontend and backend services as defined in the `docker-compose.yml` file.

**Note:** You'll need to update the backend service configuration in the `docker-compose.yml` file to match your actual backend setup.

## Project Structure

```
src/
├── app/
│   ├── core/           # Core functionality (auth, guards, services)
│   ├── features/       # Feature modules (devices, users, settings)
│   ├── shared/         # Shared components, directives, pipes
│   ├── app.component.* # Root component
│   └── app.module.ts   # Root module
├── assets/             # Static assets
├── environments/       # Environment configuration
└── index.html          # Main HTML file
```

## Authentication

The application uses JWT authentication. The token is stored in localStorage and automatically included in all API requests via an HTTP interceptor.

## Nginx Configuration

The application is served using Nginx with an optimized configuration that provides:

- **Performance Optimization**:
  - Static file caching (30 days for images, CSS, JS, fonts)
  - Gzip compression for faster content delivery
  - Optimized sendfile configuration

- **Security Enhancements**:
  - Hidden server tokens to prevent information disclosure
  - Security headers (X-Content-Type-Options, X-Frame-Options, X-XSS-Protection)
  - Restricted access to hidden files

- **SPA Support**:
  - Proper handling of Angular routing
  - HTML5 pushState support

The Nginx configuration is designed to work with any hostname and listens on port 80 by default.
