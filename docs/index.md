# OpenRDX Documentation

OpenRDX is a modern, scalable platform for managing and processing RADIUS authentication and accounting data. Built with a microservices architecture, it provides a robust solution for network access control and accounting.

## Technology Stack

- **Frontend:** Angular 16+, Material Design, TypeScript, RxJS
- **Backend:** Django 4.2+, Django REST Framework, Celery, Redis, MongoDB
- **Core Service:** Rust, Tokio, SQLx, MongoDB
- **Data Storage:** PostgreSQL, MongoDB, Redis

## Project Structure

```
.
├── core/               # Rust-based RADIUS service
├── oss_backend/        # Django REST Framework API
├── oss_frontend/       # Angular frontend application
├── k8s/               # Kubernetes configurations
├── nginx/             # Nginx reverse proxy configuration
│   ├── conf.d/        # Nginx server configurations
│   └── ssl/           # SSL certificates
├── scripts/           # Utility scripts
├── docker-compose.yml # Docker Compose configuration
└── README.md
```

## Documentation

- [Getting Started](./getting-started.md)
- [Features & Architecture](./features.md)
- [Components](./components.md)
- [Services](./services.md)
- [API Integration](./api-integration.md)
- [Testing](./testing.md)
- [Contributing](./contributing.md)

## Quick Start

For a quick start guide, see [Getting Started](./getting-started.md).

For testing the RADIUS server functionality, see [Testing](./testing.md).

---

For more details on each part of the stack, deployment, and development, use the navigation above. 