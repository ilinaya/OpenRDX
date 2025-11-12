# MS-CHAPv2 Missing Features Implementation Guide

## Overview

This document outlines the MS-CHAPv2 (Microsoft Challenge Handshake Authentication Protocol version 2) features that are currently missing from the OpenRDX RADIUS implementation and provides guidance on how to implement them for future development.

## Current Implementation Status

### ✅ Currently Implemented Features

The OpenRDX RADIUS server currently supports the following MS-CHAPv2 features:

1. **Basic Authentication Flow**
   - MS-CHAP-Challenge processing (Vendor-Specific Attribute 11)
   - MS-CHAP2-Response validation (Vendor-Specific Attribute 25)
   - NT-Response generation and verification
   - Authenticator-Response calculation
   - MS-CHAP2-Success attribute generation

2. **Cryptographic Functions**
   - NT password hashing (MD4)
   - Challenge generation using SHA-1
   - DES encryption for response generation
   - Authenticator response calculation per RFC 2759

3. **MPPE (Microsoft Point-to-Point Encryption) Support**
   - MS-MPPE-Send-Key generation (Vendor-Specific Attribute 16)
   - MS-MPPE-Recv-Key generation (Vendor-Specific Attribute 17)
   - MS-MPPE-Encryption-Policy (Vendor-Specific Attribute 7)
   - MS-MPPE-Encryption-Types (Vendor-Specific Attribute 8)
   - Session key derivation from password hash

4. **Account Status**
   - Account enabled/disabled checking
   - Basic authentication success/failure handling

### ❌ Missing Features

The following MS-CHAPv2 features are **not yet implemented** and should be considered for future development:

## 1. MS-CHAP-Error Attribute (Priority: HIGH)

### Description
The MS-CHAP-Error attribute (Vendor-Specific Attribute 2) provides detailed error information to the client when authentication fails. This is crucial for proper error handling and user experience.

### RFC Reference
- RFC 2548 Section 2.3.2
- RFC 2759 Section 6

### Current Behavior
When authentication fails, the server returns a generic Access-Reject packet without detailed error information.

### Required Implementation

#### Error Message Format
```
E=eeeeeeeeee R=r C=cccccccccccccccccccccccccccccccc V=vvvvvvvvvv M=<msg>
```

Where:
- `E` = Error code (decimal number)
- `R` = Retry flag (0 = don't retry, 1 = may retry)
- `C` = Challenge (32 hex digits, new challenge for retry)
- `V` = Password change protocol version (0 = not supported, 3 = supported)
- `M` = Human-readable error message

#### Error Codes
```rust
const ERROR_RESTRICTED_LOGON_HOURS: u32 = 646;
const ERROR_ACCT_DISABLED: u32 = 647;
const ERROR_PASSWD_EXPIRED: u32 = 648;
const ERROR_NO_DIALIN_PERMISSION: u32 = 649;
const ERROR_AUTHENTICATION_FAILURE: u32 = 691;
const ERROR_CHANGING_PASSWORD: u32 = 709;
```

#### Implementation Example

```rust
const VENDOR_ATTR_MS_CHAP_ERROR: u8 = 2;

fn create_mschapv2_error_response(
    request: &RadiusPacket,
    secret: &str,
    error_code: u32,
    retry_allowed: bool,
    password_change_supported: bool,
    message: &str,
) -> Vec<u8> {
    // Generate new challenge for retry
    let mut challenge = vec![0u8; 16];
    use rand::Rng;
    rand::thread_rng().fill(&mut challenge[..]);
    
    // Format challenge as hex string
    let challenge_hex: String = challenge.iter()
        .map(|b| format!("{:02X}", b))
        .collect();
    
    // Build error message
    let error_msg = format!(
        "E={} R={} C={} V={} M={}",
        error_code,
        if retry_allowed { "1" } else { "0" },
        challenge_hex,
        if password_change_supported { "3" } else { "0" },
        message
    );
    
    // Create Access-Reject with MS-CHAP-Error
    let mut response = RadiusPacket {
        code: 3, // Access-Reject
        identifier: request.identifier,
        length: 20,
        authenticator: request.authenticator,
        attributes: vec![
            RadiusAttribute {
                typ: ATTR_VENDOR_SPECIFIC,
                value: [
                    &VENDOR_MICROSOFT.to_be_bytes()[..],
                    &[VENDOR_ATTR_MS_CHAP_ERROR, (error_msg.len() + 2) as u8],
                    error_msg.as_bytes(),
                ].concat(),
            },
            RadiusAttribute {
                typ: ATTR_REPLY_MESSAGE,
                value: message.as_bytes().to_vec(),
            },
        ],
    };
    
    response.encode_with_response_auth(secret)
}
```

#### Usage in Authentication Flow

```rust
async fn authenticate_mschap2(&self, username: &str, peer_challenge: &[u8], nt_response: &[u8], authenticator: &[u8], secret: &str) -> Result<Mschapv2Result, sqlx::Error> {
    // ... existing code ...
    
    match result {
        Some(record) => {
            if !record.is_enabled {
                return Ok(Mschapv2Result {
                    result: AuthResult::AccountDisabled,
                    authenticator_response: None,
                    password_hash: None,
                    error_code: Some(ERROR_ACCT_DISABLED),
                    error_message: Some("Account is disabled".to_string()),
                });
            }
            
            // Check password expiry
            if let Some(expiry) = record.password_expiry {
                if expiry < chrono::Utc::now() {
                    return Ok(Mschapv2Result {
                        result: AuthResult::PasswordExpired,
                        authenticator_response: None,
                        password_hash: None,
                        error_code: Some(ERROR_PASSWD_EXPIRED),
                        error_message: Some("Password has expired".to_string()),
                    });
                }
            }
            
            // ... rest of authentication ...
        }
        None => {
            Ok(Mschapv2Result {
                result: AuthResult::InvalidPassword,
                authenticator_response: None,
                password_hash: None,
                error_code: Some(ERROR_AUTHENTICATION_FAILURE),
                error_message: Some("Invalid username or password".to_string()),
            })
        }
    }
}
```

### Benefits
- Provides clear error feedback to clients
- Enables intelligent retry logic
- Improves debugging and troubleshooting
- Better user experience

---

## 2. Password Change Support (MS-CHAP2-CPW) (Priority: HIGH)

### Description
MS-CHAP2-CPW (Change Password) allows users to change their expired or about-to-expire passwords during the authentication process without requiring administrator intervention.

### RFC Reference
- RFC 2548 Section 2.6
- RFC 2759 Section 7

### Current Behavior
Password changes are not supported. Users with expired passwords cannot authenticate.

### Required Implementation

#### New Vendor-Specific Attributes

```rust
const VENDOR_ATTR_MS_CHAP2_CPW: u8 = 27;           // Change Password v2
const VENDOR_ATTR_MS_CHAP_NT_ENC_PW: u8 = 6;       // NT-Encrypted-Password
const VENDOR_ATTR_MS_CHAP2_PASSWORD: u8 = 27;      // New Password
```

#### Database Schema Changes

```sql
-- Add to user_identifiers table
ALTER TABLE user_identifiers ADD COLUMN password_expiry TIMESTAMP;
ALTER TABLE user_identifiers ADD COLUMN password_must_change BOOLEAN DEFAULT FALSE;
ALTER TABLE user_identifiers ADD COLUMN password_last_changed TIMESTAMP;
ALTER TABLE user_identifiers ADD COLUMN password_history JSONB DEFAULT '[]';

-- Password policy table
CREATE TABLE password_policies (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    min_length INTEGER DEFAULT 8,
    max_length INTEGER DEFAULT 128,
    require_uppercase BOOLEAN DEFAULT TRUE,
    require_lowercase BOOLEAN DEFAULT TRUE,
    require_digit BOOLEAN DEFAULT TRUE,
    require_special BOOLEAN DEFAULT TRUE,
    expiry_days INTEGER DEFAULT 90,
    history_count INTEGER DEFAULT 5,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);
```

#### Password Encryption Function

```rust
fn encrypt_password_with_old_hash(
    new_password: &str,
    old_password_hash: &[u8],
) -> Vec<u8> {
    use rc4::{Rc4, KeyInit, StreamCipher};
    
    // Convert new password to UTF-16LE
    let password_utf16: Vec<u8> = new_password
        .encode_utf16()
        .flat_map(|c| c.to_le_bytes().to_vec())
        .collect();
    
    // Pad to 512 bytes (256 UTF-16 characters)
    let mut padded_password = vec![0u8; 512];
    let password_len = password_utf16.len().min(512);
    padded_password[..password_len].copy_from_slice(&password_utf16[..password_len]);
    
    // Encrypt with RC4 using old password hash as key
    let mut cipher = Rc4::new_from_slice(old_password_hash)
        .expect("Failed to create RC4 cipher");
    cipher.apply_keystream(&mut padded_password);
    
    padded_password
}
```

#### Change Password Handler

```rust
async fn handle_password_change(
    &self,
    packet: &RadiusPacket,
    secret: &str,
) -> Vec<u8> {
    // Extract MS-CHAP2-CPW attributes
    let mut encrypted_password = None;
    let mut encrypted_hash = None;
    let mut peer_challenge = None;
    let mut nt_response = None;
    let mut flags = None;
    
    for attr in &packet.attributes {
        if attr.typ == ATTR_VENDOR_SPECIFIC && attr.value.len() > 6 {
            let vendor_id = u32::from_be_bytes([
                attr.value[0], attr.value[1], 
                attr.value[2], attr.value[3]
            ]);
            
            if vendor_id == VENDOR_MICROSOFT {
                let vendor_type = attr.value[4];
                let vendor_data = &attr.value[6..];
                
                match vendor_type {
                    VENDOR_ATTR_MS_CHAP2_CPW => {
                        // Parse CPW attribute
                        if vendor_data.len() >= 516 {
                            encrypted_password = Some(&vendor_data[0..512]);
                            encrypted_hash = Some(&vendor_data[512..528]);
                            peer_challenge = Some(&vendor_data[528..544]);
                            nt_response = Some(&vendor_data[544..568]);
                            if vendor_data.len() >= 570 {
                                flags = Some(u16::from_be_bytes([
                                    vendor_data[568], 
                                    vendor_data[569]
                                ]));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    
    // Validate we have all required data
    let (enc_pw, enc_hash, peer_ch, nt_resp) = match (
        encrypted_password, encrypted_hash, peer_challenge, nt_response
    ) {
        (Some(a), Some(b), Some(c), Some(d)) => (a, b, c, d),
        _ => {
            return self.create_mschapv2_error_response(
                packet, secret,
                ERROR_CHANGING_PASSWORD,
                false, false,
                "Invalid password change request"
            );
        }
    };
    
    // Extract username
    let username = packet.attributes.iter()
        .find(|attr| attr.typ == ATTR_USER_NAME)
        .map(|attr| String::from_utf8_lossy(&attr.value).to_string())
        .unwrap_or_default();
    
    // Authenticate with old password
    let auth_result = self.authenticate_mschap2(
        &username, peer_ch, nt_resp, 
        &packet.authenticator, secret
    ).await;
    
    match auth_result {
        Ok(result) if result.result == AuthResult::Success => {
            // Decrypt new password
            let old_hash = result.password_hash.as_ref().unwrap();
            let new_password = self.decrypt_password(enc_pw, old_hash);
            
            // Validate new password
            if let Err(e) = self.validate_password(&new_password, &username).await {
                return self.create_mschapv2_error_response(
                    packet, secret,
                    ERROR_CHANGING_PASSWORD,
                    false, false,
                    &format!("Password validation failed: {}", e)
                );
            }
            
            // Update password in database
            match self.update_user_password(&username, &new_password).await {
                Ok(_) => {
                    // Return success response
                    self.create_access_accept_mschapv2(
                        packet, secret,
                        &result.authenticator_response.unwrap(),
                        0, // MS-CHAP2 identifier
                        nt_resp,
                        &nt_hash(&new_password.encode_utf16()
                            .flat_map(|c| c.to_le_bytes().to_vec())
                            .collect::<Vec<u8>>())
                    )
                }
                Err(e) => {
                    self.create_mschapv2_error_response(
                        packet, secret,
                        ERROR_CHANGING_PASSWORD,
                        false, false,
                        &format!("Failed to update password: {}", e)
                    )
                }
            }
        }
        _ => {
            self.create_mschapv2_error_response(
                packet, secret,
                ERROR_AUTHENTICATION_FAILURE,
                false, false,
                "Authentication failed during password change"
            )
        }
    }
}
```

#### Password Validation

```rust
async fn validate_password(
    &self,
    password: &str,
    username: &str,
) -> Result<(), String> {
    // Get password policy
    let policy = self.get_password_policy().await?;
    
    // Check length
    if password.len() < policy.min_length {
        return Err(format!(
            "Password must be at least {} characters",
            policy.min_length
        ));
    }
    
    if password.len() > policy.max_length {
        return Err(format!(
            "Password must be at most {} characters",
            policy.max_length
        ));
    }
    
    // Check complexity requirements
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_numeric());
    let has_special = password.chars().any(|c| !c.is_alphanumeric());
    
    if policy.require_uppercase && !has_uppercase {
        return Err("Password must contain uppercase letters".to_string());
    }
    
    if policy.require_lowercase && !has_lowercase {
        return Err("Password must contain lowercase letters".to_string());
    }
    
    if policy.require_digit && !has_digit {
        return Err("Password must contain digits".to_string());
    }
    
    if policy.require_special && !has_special {
        return Err("Password must contain special characters".to_string());
    }
    
    // Check password history
    let password_hash = nt_hash(&password.encode_utf16()
        .flat_map(|c| c.to_le_bytes().to_vec())
        .collect::<Vec<u8>>());
    
    if self.is_password_in_history(username, &password_hash).await? {
        return Err(format!(
            "Password must not match last {} passwords",
            policy.history_count
        ));
    }
    
    Ok(())
}
```

### Benefits
- Users can change expired passwords without administrator help
- Reduces help desk calls
- Improves security by enforcing password policies
- Better user experience

---

## 3. Enhanced Error Handling and Retry Logic (Priority: MEDIUM)

### Description
Implement intelligent retry logic and differentiate between temporary and permanent authentication failures.

### Current Behavior
All authentication failures are treated the same way without retry guidance.

### Required Implementation

#### Retry Logic Structure

```rust
#[derive(Debug)]
pub struct AuthFailureInfo {
    pub error_code: u32,
    pub retry_allowed: bool,
    pub retry_delay: Option<Duration>,
    pub new_challenge: Option<Vec<u8>>,
    pub message: String,
}

impl Mschapv2Result {
    pub fn from_error(
        error_type: AuthResult,
        username: &str,
    ) -> (Self, AuthFailureInfo) {
        match error_type {
            AuthResult::AccountDisabled => (
                Mschapv2Result {
                    result: error_type,
                    authenticator_response: None,
                    password_hash: None,
                },
                AuthFailureInfo {
                    error_code: ERROR_ACCT_DISABLED,
                    retry_allowed: false,
                    retry_delay: None,
                    new_challenge: None,
                    message: format!("Account {} is disabled", username),
                }
            ),
            AuthResult::InvalidPassword => (
                Mschapv2Result {
                    result: error_type,
                    authenticator_response: None,
                    password_hash: None,
                },
                AuthFailureInfo {
                    error_code: ERROR_AUTHENTICATION_FAILURE,
                    retry_allowed: true,
                    retry_delay: Some(Duration::from_secs(1)),
                    new_challenge: Some(generate_random_challenge()),
                    message: "Invalid credentials".to_string(),
                }
            ),
            // ... other cases ...
        }
    }
}
```

#### Rate Limiting

```rust
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Instant, Duration};

struct RateLimiter {
    attempts: Mutex<HashMap<String, Vec<Instant>>>,
    max_attempts: usize,
    time_window: Duration,
    lockout_duration: Duration,
}

impl RateLimiter {
    fn check_and_record(&self, username: &str) -> Result<(), AuthFailureInfo> {
        let mut attempts = self.attempts.lock().unwrap();
        let now = Instant::now();
        
        // Clean old attempts
        let user_attempts = attempts.entry(username.to_string())
            .or_insert_with(Vec::new);
        user_attempts.retain(|&t| now.duration_since(t) < self.time_window);
        
        // Check if locked out
        if user_attempts.len() >= self.max_attempts {
            let oldest_attempt = user_attempts[0];
            let unlock_time = oldest_attempt + self.lockout_duration;
            
            if now < unlock_time {
                let wait_time = unlock_time.duration_since(now);
                return Err(AuthFailureInfo {
                    error_code: ERROR_RESTRICTED_LOGON_HOURS,
                    retry_allowed: false,
                    retry_delay: Some(wait_time),
                    new_challenge: None,
                    message: format!(
                        "Account temporarily locked. Try again in {} seconds",
                        wait_time.as_secs()
                    ),
                });
            } else {
                // Lockout expired, clear attempts
                user_attempts.clear();
            }
        }
        
        // Record this attempt
        user_attempts.push(now);
        Ok(())
    }
}
```

### Benefits
- Prevents brute force attacks
- Better error reporting
- Improved security posture
- Reduced server load from repeated failed attempts

---

## 4. Password Expiry Warnings (Priority: MEDIUM)

### Description
Notify users when their passwords are about to expire, allowing them to change passwords proactively.

### Current Behavior
No warnings are provided for upcoming password expiration.

### Required Implementation

#### Warning Attribute

```rust
const VENDOR_ATTR_MS_CHAP2_SUCCESS_EXTENDED: u8 = 42; // Success with additional info
```

#### Success Message with Expiry Warning

```rust
fn create_access_accept_with_expiry_warning(
    &self,
    request: &RadiusPacket,
    secret: &str,
    auth_response: &[u8],
    days_until_expiry: i64,
    password_hash: &[u8],
    nt_response: &[u8],
) -> Vec<u8> {
    // Create normal success response
    let mut response = self.create_access_accept_mschapv2(
        request, secret, auth_response,
        0, nt_response, password_hash
    );
    
    // Add expiry warning as Reply-Message
    let warning = format!(
        "Your password will expire in {} days. Please change it soon.",
        days_until_expiry
    );
    
    // Parse and add warning attribute
    // Note: This would require modifying the response packet
    // Implementation depends on packet structure
    
    response
}
```

#### Check Password Expiry in Authentication

```rust
async fn authenticate_mschap2(&self, username: &str, peer_challenge: &[u8], nt_response: &[u8], authenticator: &[u8], secret: &str) -> Result<Mschapv2Result, sqlx::Error> {
    // ... existing authentication logic ...
    
    if authentication_successful {
        // Check password age
        if let Some(last_changed) = record.password_last_changed {
            let policy = self.get_password_policy().await?;
            let age = chrono::Utc::now().signed_duration_since(last_changed);
            let days_old = age.num_days();
            
            if days_old >= policy.expiry_days - policy.warning_days {
                let days_until_expiry = policy.expiry_days - days_old;
                
                return Ok(Mschapv2Result {
                    result: AuthResult::Success,
                    authenticator_response: Some(auth_response),
                    password_hash: Some(password_hash),
                    expiry_warning: Some(days_until_expiry),
                });
            }
        }
    }
    
    // ... rest of logic ...
}
```

### Benefits
- Reduces password expiry surprises
- Allows proactive password changes
- Reduces help desk calls
- Better user experience

---

## 5. MS-CHAPv2 Success Message Verification (Priority: LOW)

### Description
Implement proper verification of MS-CHAP2-Success messages by clients to prevent man-in-the-middle attacks.

### RFC Reference
- RFC 2759 Section 8.7

### Current Behavior
Success messages are generated but mutual authentication is not enforced.

### Required Implementation

#### Success Message Structure

The current implementation generates the authenticator response but doesn't include the proper format expected by clients:

```rust
fn create_mschapv2_success_message(
    auth_response: &[u8],
    ms_chap_ident: u8,
) -> Vec<u8> {
    // Format: "S=<auth_response_hex>"
    let auth_hex: String = auth_response.iter()
        .map(|b| format!("{:02X}", b))
        .collect();
    
    let success_str = format!("S={}", auth_hex);
    
    let mut message = vec![ms_chap_ident];
    message.extend_from_slice(success_str.as_bytes());
    
    message
}
```

### Benefits
- Prevents MITM attacks
- Ensures mutual authentication
- Improves security
- Complies with RFC 2759

---

## 6. Logging and Auditing (Priority: MEDIUM)

### Description
Comprehensive logging of MS-CHAPv2 authentication events for security auditing and troubleshooting.

### Current Behavior
Limited logging exists but not comprehensive for security auditing.

### Required Implementation

#### Audit Log Structure

```rust
#[derive(Debug, Serialize)]
struct MschapAuthEvent {
    timestamp: DateTime<Utc>,
    username: String,
    source_ip: IpAddr,
    nas_identifier: String,
    auth_method: String,
    result: String,
    error_code: Option<u32>,
    session_id: String,
    reason: String,
}

async fn log_mschap_auth_event(
    &self,
    event: MschapAuthEvent,
) -> Result<(), Box<dyn std::error::Error>> {
    // Log to database
    sqlx::query!(
        r#"
        INSERT INTO auth_audit_log 
        (timestamp, username, source_ip, nas_identifier, 
         auth_method, result, error_code, session_id, reason)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#,
        event.timestamp,
        event.username,
        event.source_ip.to_string(),
        event.nas_identifier,
        event.auth_method,
        event.result,
        event.error_code.map(|c| c as i32),
        event.session_id,
        event.reason,
    )
    .execute(self.auth_server.get_pool())
    .await?;
    
    // Also log to syslog for real-time monitoring
    info!(
        "MS-CHAPv2 Auth: user={} result={} ip={} reason={}",
        event.username, event.result, event.source_ip, event.reason
    );
    
    Ok(())
}
```

#### Database Schema

```sql
CREATE TABLE auth_audit_log (
    id SERIAL PRIMARY KEY,
    timestamp TIMESTAMP NOT NULL DEFAULT NOW(),
    username VARCHAR(255) NOT NULL,
    source_ip INET NOT NULL,
    nas_identifier VARCHAR(255),
    auth_method VARCHAR(50) NOT NULL,
    result VARCHAR(50) NOT NULL,
    error_code INTEGER,
    session_id VARCHAR(255),
    reason TEXT,
    INDEX idx_username (username),
    INDEX idx_timestamp (timestamp),
    INDEX idx_result (result)
);
```

### Benefits
- Security auditing
- Compliance requirements
- Troubleshooting
- Attack detection

---

## 7. Performance Optimizations (Priority: LOW)

### Description
Various performance optimizations for high-volume MS-CHAPv2 authentication.

### Recommendations

1. **Password Hash Caching**
   - Cache NT password hashes to avoid repeated MD4 calculations
   - Implement TTL-based cache invalidation
   - Use Redis for distributed deployments

2. **Connection Pooling**
   - Already implemented with SQLx
   - Ensure optimal pool sizing

3. **Async Processing**
   - Already using Tokio async runtime
   - Consider batch processing for audit logs

4. **Challenge Generation**
   - Use cryptographically secure random number generator
   - Consider pre-generating challenge pool

```rust
use rand::rngs::OsRng;
use rand::RngCore;

fn generate_secure_challenge() -> Vec<u8> {
    let mut challenge = vec![0u8; 16];
    OsRng.fill_bytes(&mut challenge);
    challenge
}
```

---

## Implementation Priority Summary

### Phase 1 (High Priority)
1. ✅ Implement MS-CHAP-Error attribute
2. ✅ Add password change support (MS-CHAP2-CPW)
3. ✅ Enhanced error handling and retry logic

### Phase 2 (Medium Priority)
1. ✅ Password expiry warnings
2. ✅ Comprehensive logging and auditing
3. ✅ Rate limiting and account lockout

### Phase 3 (Low Priority)
1. ✅ Success message verification enhancement
2. ✅ Performance optimizations
3. ✅ Additional security hardening

---

## Testing Recommendations

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_message_format() {
        let error_msg = format_mschapv2_error(
            ERROR_AUTHENTICATION_FAILURE,
            true,
            &[0u8; 16],
            false,
            "Invalid password"
        );
        
        assert!(error_msg.starts_with("E=691"));
        assert!(error_msg.contains("R=1"));
    }

    #[test]
    fn test_password_encryption() {
        let new_password = "NewP@ssw0rd123";
        let old_hash = nt_hash(b"OldPassword");
        
        let encrypted = encrypt_password_with_old_hash(
            new_password,
            &old_hash
        );
        
        assert_eq!(encrypted.len(), 512);
    }

    #[tokio::test]
    async fn test_password_validation() {
        let validator = PasswordValidator::new();
        
        // Valid password
        assert!(validator.validate("ValidP@ss123", "testuser").await.is_ok());
        
        // Too short
        assert!(validator.validate("P@ss1", "testuser").await.is_err());
        
        // No special char
        assert!(validator.validate("Password123", "testuser").await.is_err());
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_mschapv2_with_expired_password() {
    // Setup test user with expired password
    let user = create_test_user_with_expired_password().await;
    
    // Attempt authentication
    let result = radius_server.authenticate_mschap2(
        &user.username,
        &peer_challenge,
        &nt_response,
        &authenticator,
        "secret"
    ).await;
    
    // Should receive password expired error
    assert!(matches!(result.result, AuthResult::PasswordExpired));
    assert_eq!(result.error_code, Some(ERROR_PASSWD_EXPIRED));
}

#[tokio::test]
async fn test_password_change_flow() {
    let user = create_test_user().await;
    let old_password = "OldP@ssw0rd123";
    let new_password = "NewP@ssw0rd456";
    
    // Create password change request
    let cpw_request = create_mschapv2_cpw_request(
        &user.username,
        old_password,
        new_password
    );
    
    // Process request
    let response = radius_server.handle_password_change(
        &cpw_request,
        "secret"
    ).await;
    
    // Should receive Access-Accept
    assert_eq!(response[0], 2); // Access-Accept code
    
    // Verify new password works
    let auth_result = authenticate_with_password(
        &user.username,
        new_password
    ).await;
    assert!(auth_result.is_ok());
}
```

---

## Security Considerations

1. **Secure Random Number Generation**
   - Always use cryptographically secure RNG for challenges
   - Use `OsRng` or similar CSPRNG

2. **Password Storage**
   - Never store plaintext passwords (current implementation uses plain_password - should migrate)
   - Store NT hashes if MS-CHAP authentication is required
   - Consider salted hashes for additional security

3. **Rate Limiting**
   - Implement per-user and per-IP rate limiting
   - Use exponential backoff for repeated failures

4. **Audit Logging**
   - Log all authentication attempts
   - Include source IP, timestamp, and result
   - Protect audit logs from tampering

5. **TLS/RadSec**
   - Prefer RadSec over traditional RADIUS when possible
   - Ensures confidentiality of authentication exchange

---

## References

### RFCs and Standards

- **RFC 2759**: Microsoft PPP CHAP Extensions, Version 2 (MS-CHAPv2)
  - https://tools.ietf.org/html/rfc2759

- **RFC 2548**: Microsoft Vendor-specific RADIUS Attributes
  - https://tools.ietf.org/html/rfc2548

- **RFC 3078**: Microsoft Point-To-Point Encryption (MPPE) Protocol
  - https://tools.ietf.org/html/rfc3078

- **RFC 3079**: Deriving Keys for use with MPPE
  - https://tools.ietf.org/html/rfc3079

- **RFC 2865**: Remote Authentication Dial In User Service (RADIUS)
  - https://tools.ietf.org/html/rfc2865

### Additional Resources

- Microsoft TechNet: Understanding MS-CHAP v2
- FreeRADIUS MS-CHAP documentation
- Wireshark RADIUS dissector (for testing and debugging)

---

## Conclusion

This document provides a comprehensive guide for implementing the missing MS-CHAPv2 features in OpenRDX. The features are prioritized based on their importance for security, user experience, and standards compliance.

The implementation should be done in phases, starting with high-priority features (error handling and password change support) and progressing to medium and low-priority enhancements.

Each feature includes detailed implementation guidance, code examples, and testing recommendations to ensure a robust and secure implementation.

For questions or clarifications, please refer to the RFC specifications or consult with the development team.
