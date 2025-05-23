# RADIUS Server Implementation

## Overview
The RADIUS server implementation supports multiple authentication methods including PAP, CHAP, MS-CHAP, and MS-CHAPv2. It is designed to work with the existing user authentication system and provides secure authentication for network access.

## Supported Authentication Methods

### PAP (Password Authentication Protocol)
- Simple password-based authentication
- Passwords are encrypted using the RADIUS shared secret
- Uses MD5 for password encryption

### CHAP (Challenge Handshake Authentication Protocol)
- Challenge-response based authentication
- Uses MD5 for response generation
- More secure than PAP as passwords are never sent in clear text
- Challenge is generated from the RADIUS authenticator

### MS-CHAP (Microsoft Challenge Handshake Authentication Protocol)
- Microsoft's implementation of CHAP
- Uses DES encryption for challenge-response
- Supports UTF-16LE password encoding
- Implements proper DES key setup with parity bits

### MS-CHAPv2 (Microsoft Challenge Handshake Authentication Protocol Version 2)
- Enhanced version of MS-CHAP
- Uses SHA1 for challenge hash generation
- Implements proper NT hash generation
- Supports peer challenge for mutual authentication
- Uses DES encryption for response generation

## Implementation Details

### Authentication Flow
1. Server receives Access-Request packet
2. Authentication method is detected from packet attributes
3. Appropriate authentication handler is called based on method
4. User credentials are verified against the database
5. Access-Accept or Access-Reject is sent based on authentication result

### Security Features
- Support for RADIUS shared secrets
- Message-Authenticator attribute for packet integrity
- Proper handling of vendor-specific attributes
- Secure password storage and verification
- Support for account status checking

### Database Integration
- Uses the existing user_identifiers table
- Supports plain password storage for authentication
- Checks account enabled status
- Maintains user identifier types

## Configuration

### Required Dependencies
```toml
[dependencies]
tokio = { version = "1.0", features = ["full"] }
sqlx = { version = "0.8.6", features = ["runtime-tokio-rustls", "postgres", "json", "chrono", "ipnetwork"] }
md5 = "0.10.6"
hmac = "0.12.1"
digest = "0.10.7"
des = "0.8.0"
sha1 = "0.10.6"
md4 = "0.10"
generic-array = "0.14"  # Required for DES encryption
```

### Environment Variables
- `RADIUS_BIND_ADDR`: The address and port to bind the RADIUS server to
- `RADIUS_SECRET`: The shared secret for RADIUS communication

### Testing
The project includes shell scripts in `core/tests/` for testing different authentication methods:

- `test_pap_radius.sh`: Tests PAP authentication
- `test_chap_radius.sh`: Tests CHAP authentication
- `test_mschap_radius.sh`: Tests MS-CHAP authentication

These scripts simulate RADIUS client behavior and can be used to verify the server's authentication functionality.

## Usage Example

```rust
let auth_server = Arc::new(AuthServer::new(pool).await?);
let radius_server = RadiusAuthServer::new("0.0.0.0:1812".to_string(), auth_server).await?;
radius_server.run().await?;
```

## Protocol Support

### RADIUS Attributes
- User-Name (1)
- User-Password (2)
- CHAP-Password (3)
- Reply-Message (18)
- Vendor-Specific (26)
- MS-CHAP-Challenge (11)
- MS-CHAP-Response (1)
- MS-CHAP2-Response (25)

### Vendor-Specific Attributes
- Microsoft Vendor ID: 311
- MS-CHAP-Challenge: 11
- MS-CHAP-Response: 1
- MS-CHAP2-Response: 25
- MS-CHAP2-Challenge: 11

## Security Considerations
1. Always use strong shared secrets
2. Implement proper network security (firewall rules)
3. Monitor authentication attempts
4. Regularly rotate shared secrets
5. Use secure password storage
6. Implement proper error handling and logging

## Troubleshooting
- Check server logs for authentication failures
- Verify shared secrets match between client and server
- Ensure proper network connectivity
- Verify user credentials in database
- Check account status (enabled/disabled) 