# Services

This document provides detailed information about the services used in the OSS Frontend application.

## Core Services

### AuthService
Handles authentication and authorization.

**Features:**
- User login/logout
- Token management
- Session handling
- Permission checking
- Role-based access control

### HttpService
Base HTTP service for API communication.

**Features:**
- Request/response interceptors
- Error handling
- Token injection
- Request caching
- Retry logic

### NotificationService
Manages application notifications.

**Features:**
- Success messages
- Error messages
- Warning messages
- Info messages
- Toast notifications

## Feature Services

### Device Services

#### NasService
Manages NAS device operations.

**Features:**
- CRUD operations for NAS devices
- Group management
- Secret management
- CoA configuration
- Vendor management

#### SwitchService
Manages switch operations.

**Features:**
- CRUD operations for switches
- Group management
- Configuration management
- Status monitoring

### User Services

#### UserService
Manages user operations.

**Features:**
- CRUD operations for users
- Role management
- Permission management
- Group assignment
- Profile management

#### AdminService
Manages admin operations.

**Features:**
- CRUD operations for admins
- Role management
- Permission management
- Group assignment
- Profile management

### Group Services

#### GroupService
Manages group operations.

**Features:**
- CRUD operations for groups
- Member management
- Permission configuration
- Role assignment

#### AdminGroupService
Manages admin group operations.

**Features:**
- CRUD operations for admin groups
- Member management
- Permission configuration
- Role assignment

## Utility Services

### StorageService
Manages local storage operations.

**Features:**
- Data persistence
- Cache management
- Session storage
- Local storage
- Cookie management

### LoggerService
Handles application logging.

**Features:**
- Error logging
- Debug logging
- Info logging
- Warning logging
- Log persistence

### ConfigService
Manages application configuration.

**Features:**
- Environment configuration
- Feature flags
- API endpoints
- Application settings
- Theme configuration

## Service Architecture

### Service Structure
```
src/app/
├── core/
│   ├── services/
│   │   ├── auth.service.ts
│   │   ├── http.service.ts
│   │   └── notification.service.ts
├── features/
│   ├── devices/
│   │   ├── services/
│   │   │   ├── nas.service.ts
│   │   │   └── switch.service.ts
│   ├── users/
│   │   ├── services/
│   │   │   ├── user.service.ts
│   │   │   └── admin.service.ts
└── shared/
    ├── services/
    │   ├── storage.service.ts
    │   ├── logger.service.ts
    │   └── config.service.ts
```

### Service Communication
- Dependency injection
- Event emitters
- Subject/Observable patterns
- State management

### Error Handling
- Global error handling
- Service-specific error handling
- Error logging
- Error recovery

## Best Practices

### Service Design
- Single responsibility
- Dependency injection
- Interface-based design
- Error handling
- Logging

### Performance
- Request caching
- Response caching
- Lazy loading
- Resource optimization

### Security
- Token management
- Request validation
- Response validation
- Error handling
- Logging

### Testing
- Unit tests
- Integration tests
- Mock services
- Test coverage

## Core Service (Rust)
- RADIUS authentication and accounting
- Stores accounting data in MongoDB
- Communicates with backend via message broker

## Backend Service (Django)
- RESTful API for user, device, and system management
- Integrates with PostgreSQL and MongoDB
- Handles authentication, authorization, and configuration

## Frontend Service (Angular)
- User interface for managing users, devices, and settings
- Real-time updates and notifications
- Communicates with backend API

## Nginx Reverse Proxy
- SSL/TLS termination
- Routing and load balancing
- Security and CORS headers

## PostgreSQL
- Relational data storage for users, configs, and system state

## MongoDB
- Storage for RADIUS accounting and logs

## Redis
- Caching, message broker, and session storage 