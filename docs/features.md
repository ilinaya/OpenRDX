# Features & Architecture

## Architecture Overview

OpenRDX is built with a microservices architecture, consisting of:
- **Core Service (Rust):** High-performance RADIUS server, handles authentication/accounting, stores data in MongoDB.
- **Backend Service (Django):** RESTful API, user management, system configuration, integrates with PostgreSQL and MongoDB.
- **Frontend Service (Angular):** Modern SPA, Material Design, real-time updates, served via Nginx.
- **Nginx Reverse Proxy:** SSL/TLS termination, routing, CORS, security headers.
- **PostgreSQL:** Relational data, user/config storage.
- **MongoDB:** RADIUS accounting, logging, analytics.
- **Redis:** Caching, message broker, session storage.

## Security Features
- SSL/TLS encryption for all HTTP traffic
- Automatic HTTP to HTTPS redirection
- CORS headers for API access
- Internal services not exposed to public
- Secure SSL configuration (TLSv1.2, TLSv1.3)
- Proper file permissions for SSL certificates
- JWT-based authentication
- Rate limiting and DDoS protection

## Main Features
- Modern, scalable RADIUS management
- User and group management
- Device (NAS) management
- Real-time updates and notifications
- RESTful API for integration
- Kubernetes and Docker support
- Multi-database support (PostgreSQL, MongoDB, Redis)
- Secure by default

For a detailed breakdown of components and services, see [Components](./components.md) and [Services](./services.md).

## Device Management

### NAS Devices
- View list of NAS devices
- Add new NAS devices
- Edit existing NAS devices
- Delete NAS devices
- Configure CoA settings
- Manage NAS groups
- Assign secrets to NAS devices

### Switches
- View list of switches
- Add new switches
- Edit existing switches
- Delete switches
- Configure switch settings
- Manage switch groups

## User Management

### Admin Users
- View list of admin users
- Add new admin users
- Edit existing admin users
- Delete admin users
- Manage admin user permissions
- Assign users to admin groups

### Regular Users
- View list of users
- Add new users
- Edit existing users
- Delete users
- Manage user permissions
- Assign users to groups

## Group Management

### Admin Groups
- View list of admin groups
- Create new admin groups
- Edit existing admin groups
- Delete admin groups
- Manage group members
- Configure group permissions

### User Groups
- View list of user groups
- Create new user groups
- Edit existing user groups
- Delete user groups
- Manage group members
- Configure group permissions

## Settings

### System Settings
- Configure system-wide settings
- Manage application preferences
- Set up email notifications
- Configure security settings

### Vendor Management
- View list of vendors
- Add new vendors
- Edit existing vendors
- Delete vendors
- Manage vendor-specific settings

## Authentication and Authorization

### Login
- User authentication
- Remember me functionality
- Password reset
- Two-factor authentication

### Authorization
- Role-based access control
- Permission management
- Session management
- Access token handling

## Internationalization

- Multi-language support
- Language switching
- Date and time formatting
- Number formatting
- Currency formatting

## UI/UX Features

### Responsive Design
- Mobile-friendly interface
- Adaptive layouts
- Touch-friendly controls

### Theme Support
- Light/Dark mode
- Custom theme colors
- High contrast mode

### Accessibility
- Screen reader support
- Keyboard navigation
- ARIA labels
- Focus management

## Data Management

### Data Tables
- Sorting
- Filtering
- Pagination
- Export functionality

### Forms
- Validation
- Error handling
- Dynamic fields
- File uploads

## Notifications

### System Notifications
- Success messages
- Error messages
- Warning messages
- Info messages

### User Notifications
- In-app notifications
- Email notifications
- Push notifications

## Search and Filter

- Global search
- Advanced filtering
- Search history
- Saved searches

## Reporting

- Generate reports
- Export data
- Custom report templates
- Scheduled reports 