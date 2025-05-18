# Components

This document provides detailed information about the components used in the OSS Frontend application.

## Core Components

### AppComponent
The root component that bootstraps the application.

**Features:**
- Main layout structure
- Navigation menu
- Authentication state management
- Theme switching
- Language switching

### HeaderComponent
The main header component that appears on all pages.

**Features:**
- User menu
- Notifications
- Search bar
- Language selector
- Theme toggle

### SidebarComponent
The main navigation sidebar.

**Features:**
- Navigation menu
- Collapsible sections
- Active route highlighting
- Permission-based menu items

## Feature Components

### Device Management

#### NasListComponent
Displays a list of NAS devices.

**Features:**
- Data table with sorting and filtering
- Pagination
- Search functionality
- Bulk actions
- Export options

#### NasFormComponent
Form for creating and editing NAS devices.

**Features:**
- Form validation
- Dynamic fields
- Vendor selection
- Secret management
- Group assignment

#### SwitchListComponent
Displays a list of switches.

**Features:**
- Data table with sorting and filtering
- Pagination
- Search functionality
- Bulk actions
- Export options

### User Management

#### UserListComponent
Displays a list of users.

**Features:**
- Data table with sorting and filtering
- Pagination
- Search functionality
- Bulk actions
- Export options

#### UserFormComponent
Form for creating and editing users.

**Features:**
- Form validation
- Role selection
- Group assignment
- Permission management

### Group Management

#### GroupListComponent
Displays a list of groups.

**Features:**
- Data table with sorting and filtering
- Pagination
- Search functionality
- Bulk actions
- Export options

#### GroupFormComponent
Form for creating and editing groups.

**Features:**
- Form validation
- Member management
- Permission configuration
- Role assignment

## Shared Components

### DataTableComponent
Reusable data table component.

**Features:**
- Sorting
- Filtering
- Pagination
- Selection
- Custom actions
- Export functionality

### FormFieldComponent
Reusable form field component.

**Features:**
- Input validation
- Error messages
- Custom styling
- Accessibility support
- Internationalization

### ModalComponent
Reusable modal dialog component.

**Features:**
- Custom content
- Size options
- Animation
- Backdrop
- Keyboard navigation

### NotificationComponent
Reusable notification component.

**Features:**
- Multiple types (success, error, warning, info)
- Auto-dismiss
- Custom duration
- Stacking
- Animation

## Utility Components

### LoadingSpinnerComponent
Displays a loading spinner.

**Features:**
- Custom size
- Custom color
- Overlay option
- Text display

### ErrorMessageComponent
Displays error messages.

**Features:**
- Multiple error types
- Custom styling
- Dismissible
- Animation

### ConfirmationDialogComponent
Displays confirmation dialogs.

**Features:**
- Custom message
- Custom buttons
- Keyboard support
- Animation

## Component Architecture

### Component Structure
```
src/app/
├── core/
│   ├── components/
│   │   ├── header/
│   │   ├── sidebar/
│   │   └── footer/
├── features/
│   ├── devices/
│   │   ├── components/
│   │   │   ├── nas-list/
│   │   │   ├── nas-form/
│   │   │   ├── switch-list/
│   │   │   └── switch-form/
│   ├── users/
│   │   ├── components/
│   │   │   ├── user-list/
│   │   │   └── user-form/
└── shared/
    ├── components/
    │   ├── data-table/
    │   ├── form-field/
    │   └── modal/
```

### Component Communication
- Input/Output decorators
- Services
- State management
- Event bus

### Component Lifecycle
- OnInit
- OnDestroy
- OnChanges
- AfterViewInit
- AfterContentInit

## Best Practices

### Component Design
- Single responsibility
- Reusability
- Maintainability
- Testability

### Performance
- Change detection strategy
- Lazy loading
- Memory management
- Resource optimization

### Accessibility
- ARIA attributes
- Keyboard navigation
- Screen reader support
- Focus management

### Testing
- Unit tests
- Integration tests
- E2E tests
- Test coverage

## Core Service (Rust)
- High-performance RADIUS server
- Handles authentication and accounting
- Exposes UDP ports 1812, 1813
- Stores accounting data in MongoDB

## Backend Service (Django)
- RESTful API with Django REST Framework
- User management and system configuration
- Integrates with PostgreSQL and MongoDB
- Health check endpoint for monitoring

## Frontend Service (Angular)
- Modern single-page application
- Material Design components
- Real-time updates using WebSocket
- Served through Nginx

## Nginx Reverse Proxy
- Handles all HTTP/HTTPS traffic
- SSL/TLS termination
- Routes traffic to services
- CORS and security headers

## PostgreSQL
- Primary relational database
- Stores user data, configurations, and system state

## MongoDB
- Specialized storage for RADIUS accounting
- High-performance logging and analytics

## Redis
- Caching layer
- Message broker for Celery
- Session storage 