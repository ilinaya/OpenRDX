# Django REST Framework API

A Django REST Framework API with JWT authentication, Swagger documentation, and multiple apps for managing users, NAS devices, and settings.

## Features

- JWT authentication
- Swagger API documentation
- Admin users management with email invitations and password reset
- Regular users management with group support
- NAS (Network Access Server) management with group support
- Excel template download and bulk import:
  - **User Groups**: Download template and upload Excel files to bulk import groups
  - **Users**: Download template with 3 sheets (Users, Identifiers, NAS Authorizations) and upload to bulk import users with their identifiers and NAS authorizations
  - **NAS Groups**: Download template and upload Excel files to bulk import groups
  - **NAS Devices**: Download template and upload Excel files to bulk import devices
- System settings management
- Accounting data from MongoDB for session tracking
- RADIUS authentication attribute management
- User-NAS relationship management with attribute groups
- Change of Authorization (CoA) via Redis for dynamic policy changes
- Email integration with Mailgun
- Google Chat webhook integration
- Dockerized deployment

## Project Structure

The project consists of the following apps:

- **admin_users**: Management of admin users who can access the Django admin interface
- **authentication**: JWT token-based authentication
- **nas**: Management of Network Access Server (NAS) devices
- **settings_app**: Management of system settings
- **users**: Management of regular users with group support
- **accounting**: Retrieval of accounting session data from MongoDB
- **radius**: RADIUS authentication attributes and user-NAS relationships

## Environment Variables

The application uses the following environment variables:

### Django Settings
- `DEBUG`: Enable debug mode (default: False)
- `SECRET_KEY`: Django secret key for security
- `ALLOWED_HOSTS`: Comma-separated list of allowed hosts
- `LOGGING_LEVEL`: Logging level (default: INFO)
- `DJANGO_SETTINGS_MODULE`: Path to settings module (default: core.settings)

### Database Settings
- `DB_NAME`: PostgreSQL database name (default: postgres)
- `DB_USER`: PostgreSQL username (default: postgres)
- `DB_PASSWORD`: PostgreSQL password (default: postgres)
- `DB_HOST`: PostgreSQL host (default: db)
- `DB_PORT`: PostgreSQL port (default: 5432)
- `MONGODB_URI`: MongoDB connection URI (default: mongodb://mongodb:27017/radius_accounting)

### JWT Settings
- `JWT_SECRET_KEY`: Secret key for JWT tokens
- `JWT_ACCESS_TOKEN_LIFETIME`: Lifetime of access tokens in minutes (default: 60)
- `JWT_REFRESH_TOKEN_LIFETIME`: Lifetime of refresh tokens in days (default: 1)

### Email Settings (Mailgun)
- `MAILGUN_API_KEY`: Mailgun API key
- `MAILGUN_SENDER_DOMAIN`: Mailgun sender domain (default: mg.example.com)
- `DEFAULT_FROM_EMAIL`: Default sender email (default: noreply@example.com)

### Integrations
- `GOOGLE_CHAT_WEBHOOK_URL`: Google Chat webhook URL for notifications

### Redis Settings
- `REDIS_HOST`: Redis server hostname (default: localhost)
- `REDIS_PORT`: Redis server port (default: 6379)
- `REDIS_DB`: Redis database number (default: 0)
- `COA_TOPIC`: Redis topic for Change of Authorization messages (default: radius_coa)

## Setup

### Prerequisites

- Docker and Docker Compose
- Python 3.10+ (for local development)

### Using Docker

1. Clone the repository
2. Copy `.env.example` to `.env` and update the values
3. Build and run the Docker container:

```bash
# Build the Docker image
docker build -t oss-backend .

# Run the container with environment variables from .env file
docker run -d --name oss-backend -p 8000:8000 --env-file .env oss-backend
```

4. Access the API at http://localhost:8000/
5. Access the Swagger documentation at http://localhost:8000/swagger/

### Local Development

1. Clone the repository
2. Create a virtual environment:

```bash
python -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate
```

3. Install dependencies:

```bash
pip install -r requirements.txt
```

4. Copy `.env.example` to `.env` and update the values
5. Run migrations:

```bash
python manage.py migrate
```

6. Create a superuser:

```bash
python manage.py createsuperuser
```

7. Run the development server:

```bash
python manage.py runserver
```

8. Access the API at http://localhost:8000/
9. Access the Swagger documentation at http://localhost:8000/swagger/

## API Endpoints

### Authentication

- `POST /api/auth/token/`: Obtain JWT token
- `POST /api/auth/token/refresh/`: Refresh JWT token

### Admin Users

- `GET /api/admin-users/`: List all admin users
- `POST /api/admin-users/`: Create a new admin user
- `GET /api/admin-users/{id}/`: Retrieve an admin user
- `PUT /api/admin-users/{id}/`: Update an admin user
- `DELETE /api/admin-users/{id}/`: Delete an admin user
- `POST /api/admin-users/{id}/activate/`: Activate an admin user
- `POST /api/admin-users/{id}/deactivate/`: Deactivate an admin user
- `POST /api/admin-users/{id}/send-invitation/`: Send an invitation email
- `POST /api/admin-users/{id}/send-password-reset/`: Send a password reset email
- `POST /api/admin-users/set-password/`: Set password using invitation token
- `POST /api/admin-users/reset-password/`: Reset password using reset token

### Users

- `GET /api/users/`: List all users
- `POST /api/users/`: Create a new user
- `GET /api/users/{id}/`: Retrieve a user
- `PUT /api/users/{id}/`: Update a user
- `DELETE /api/users/{id}/`: Delete a user
- `GET /api/users/download_template/`: Download Excel template for importing users with identifiers and NAS authorizations
- `POST /api/users/upload_excel/`: Upload Excel file to bulk import users, identifiers, and NAS authorizations
- `GET /api/users/groups/`: List all user groups
- `POST /api/users/groups/`: Create a new user group
- `GET /api/users/groups/{id}/`: Retrieve a user group
- `PUT /api/users/groups/{id}/`: Update a user group
- `DELETE /api/users/groups/{id}/`: Delete a user group
- `GET /api/users/groups/tree/`: Get user groups as a tree structure
- `GET /api/users/groups/download_template/`: Download Excel template for importing user groups
- `POST /api/users/groups/upload_excel/`: Upload Excel file to bulk import user groups

### NAS (Network Access Server)

- `GET /api/nas/nas/`: List all NAS devices
- `POST /api/nas/nas/`: Register a new NAS device
- `GET /api/nas/nas/{id}/`: Retrieve a NAS device
- `PUT /api/nas/nas/{id}/`: Update a NAS device
- `DELETE /api/nas/nas/{id}/`: Delete a NAS device
- `GET /api/nas/nas/by-group/`: Filter NAS devices by group
- `GET /api/nas/nas/download_template/`: Download Excel template for importing NAS devices
- `POST /api/nas/nas/upload_excel/`: Upload Excel file to bulk import NAS devices
- `GET /api/nas/groups/`: List all NAS groups
- `POST /api/nas/groups/`: Create a new NAS group
- `GET /api/nas/groups/{id}/`: Retrieve a NAS group
- `PUT /api/nas/groups/{id}/`: Update a NAS group
- `DELETE /api/nas/groups/{id}/`: Delete a NAS group
- `GET /api/nas/groups/tree/`: Get NAS groups as a tree structure
- `GET /api/nas/groups/download_template/`: Download Excel template for importing NAS groups
- `POST /api/nas/groups/upload_excel/`: Upload Excel file to bulk import NAS groups

### Settings

- `GET /api/settings/`: List all settings
- `POST /api/settings/`: Create a new setting
- `GET /api/settings/{id}/`: Retrieve a setting
- `PUT /api/settings/{id}/`: Update a setting
- `DELETE /api/settings/{id}/`: Delete a setting

### Accounting

- `GET /api/accounting/sessions/nas/`: Get paginated accounting sessions for a specific NAS device
- `GET /api/accounting/sessions/user/`: Get paginated accounting sessions for a specific user

### RADIUS Authentication

- `GET /api/radius/attribute-groups/`: List all authentication attribute groups
- `POST /api/radius/attribute-groups/`: Create a new authentication attribute group
- `GET /api/radius/attribute-groups/{id}/`: Retrieve an authentication attribute group
- `PUT /api/radius/attribute-groups/{id}/`: Update an authentication attribute group
- `DELETE /api/radius/attribute-groups/{id}/`: Delete an authentication attribute group (system groups cannot be deleted)

- `GET /api/radius/attributes/`: List all RADIUS attributes
- `POST /api/radius/attributes/`: Create a new RADIUS attribute
- `GET /api/radius/attributes/{id}/`: Retrieve a RADIUS attribute
- `PUT /api/radius/attributes/{id}/`: Update a RADIUS attribute
- `DELETE /api/radius/attributes/{id}/`: Delete a RADIUS attribute
- `GET /api/radius/attributes/by-group/`: Filter attributes by group ID

- `GET /api/radius/user-nas/`: List all user-NAS relationships
- `POST /api/radius/user-nas/`: Create a new user-NAS relationship
- `GET /api/radius/user-nas/{id}/`: Retrieve a user-NAS relationship
- `PUT /api/radius/user-nas/{id}/`: Update a user-NAS relationship
- `DELETE /api/radius/user-nas/{id}/`: Delete a user-NAS relationship
- `GET /api/radius/user-nas/by-user/`: Filter relationships by user ID
- `GET /api/radius/user-nas/by-nas/`: Filter relationships by NAS ID
- `POST /api/radius/user-nas/{id}/change-attribute-group/`: Change the attribute group for a user-NAS relationship
- `POST /api/radius/user-nas/{id}/reauth/`: Trigger a reauthentication for a user-NAS relationship

## License

This project is licensed under the MIT License.
