# System Diagrams

This directory contains PlantUML diagrams that visualize various aspects of the OpenRDX system.

## Available Diagrams

1. **system-architecture.puml** - Overall system architecture and components
2. **radius-authentication-flow.puml** - RADIUS authentication sequence diagram
3. **component-interaction.puml** - Frontend component interactions
4. **database-schema.puml** - Database schema and relationships
5. **deployment-architecture.puml** - Docker Compose deployment architecture
6. **authentication-protocols.puml** - Authentication protocol flows
7. **northbound-api-architecture.puml** - Northbound API architecture and components
8. **northbound-api-authentication-flow.puml** - API key authentication flow for northbound API
9. **northbound-api-endpoints.puml** - Northbound API endpoint structure

## Viewing Diagrams

You can view these diagrams using:

1. **PlantUML Tools**:
   - VS Code extension: "PlantUML" by jebbs
   - IntelliJ/WebStorm plugin: PlantUML integration
   - Online viewer: http://www.plantuml.com/plantuml/uml/

2. **VS Code**:
   - Install the PlantUML extension
   - Open any `.puml` file
   - Press `Alt+D` (or `Cmd+D` on Mac) to preview

3. **Online**:
   - Copy the content of a `.puml` file
   - Paste it into http://www.plantuml.com/plantuml/uml/
   - View the rendered diagram

## Generating Images

To generate PNG or SVG images from the diagrams:

```bash
# Install PlantUML (requires Java)
# On macOS:
brew install plantuml

# On Ubuntu/Debian:
sudo apt-get install plantuml

# Generate images
cd docs/diagrams
plantuml *.puml
```

This will generate PNG images for each diagram file.

## Diagram Descriptions

### System Architecture
Shows the overall system components including frontend, backend, core services, databases, and networking.

### RADIUS Authentication Flow
Sequence diagram showing the flow of a RADIUS authentication request from client to server.

### Component Interaction
Frontend component interactions and data flow in the Angular application.

### Database Schema
Entity-relationship diagram showing all database models and their relationships.

### Deployment Architecture
Docker Compose deployment showing all services, networks, and volumes.

### Authentication Protocols
Various RADIUS authentication protocols (PAP, CHAP, MS-CHAP, MS-CHAPv2).

### Northbound API Architecture
Architecture of the northbound API showing:
- Direct PostgreSQL database connection via deadpool-postgres connection pooling
- JWT authentication middleware
- Connection pool management with exponential backoff and automatic reconnection
- All database tables accessed (Users, NAS, NAS Groups, Vendors, Secrets, etc.)
- OpenAPI/Swagger documentation generation via utoipa

### Northbound API Authentication Flow
Sequence diagram showing:
- API key generation process from the backend
- JWT token verification flow
- Database query execution for both read and write operations
- Connection pool management during requests
- Transaction handling for create/update operations

### Northbound API Endpoints
Structure of all available endpoints in the northbound API:
- **Public Endpoints**: Health check, Swagger UI, OpenAPI spec
- **Users**: Full CRUD operations (GET, POST, PUT, DELETE) with pagination, identities, and groups
- **User Groups**: Full CRUD operations for hierarchical user group management (GET, POST, PUT, DELETE)
- **User Identifiers**: Update operation (PUT) for user identifier modifications
- **User Identifier Types**: List all available identifier types (GET)
- **NAS Groups**: Full CRUD operations for hierarchical NAS group management
- **NAS Devices**: Full CRUD operations with secrets, vendors, groups, and timezones
- **Vendors**: List all available vendors
- **Secrets**: List all available secrets

### Excel Template Import/Export
The system provides Excel template download and upload functionality through the Django backend:
- **User Groups**: Download template (Name, Description, Parent Group Name, Allow Any NAS) and upload to bulk import groups
- **Users**: Download template with 3 sheets (Users, Identifiers, NAS Authorizations) and upload to bulk import users with their identifiers and NAS authorizations
- **NAS Groups**: Download template (Name, Description, Parent Group Name) and upload to bulk import groups
- **NAS Devices**: Download template (Name, IP Address, Description, Vendor ID, Secret ID, Timezone ID, CoA Enabled, CoA Port, Group Names, Is Active) and upload to bulk import devices
- All templates include example rows and proper validation
- Upload endpoints validate data and provide detailed error messages

