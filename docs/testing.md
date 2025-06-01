# Testing

This document describes the testing tools and scripts available for testing the RADIUS server functionality.

## Prerequisites

Before running the tests, ensure you have the following installed:
- FreeRADIUS client tools (for radtest)
  ```bash
  # Ubuntu/Debian
  sudo apt-get install freeradius-utils
  
  # CentOS/RHEL
  sudo yum install freeradius-utils
  
  # macOS
  brew install freeradius
  ```

## Test Scripts

### Authentication Testing

The `test_radius_auth.sh` script tests the RADIUS authentication server functionality. It includes tests for:

- Basic authentication
- Different authentication methods (PAP, CHAP, MS-CHAP)
- Invalid credentials
- Missing attributes
- Different NAS ports

To run the authentication tests:
```bash
./core/test_radius_auth.sh
```

### Accounting Testing

The `test_radius_acct.sh` script tests the RADIUS accounting server functionality. It includes tests for:

- Accounting Start
- Accounting Interim Update
- Accounting Stop
- Accounting On/Off
- Invalid accounting packets
- Different NAS ports

To run the accounting tests:
```bash
./core/test_radius_acct.sh
```

## Test Configuration

Both test scripts use the following default configuration:
- Server: localhost
- Auth Port: 1812
- Acct Port: 1813
- Secret: testing123
- Username: testuser
- Password: testpass

You can modify these values in the scripts if needed.

## Test Output

The test scripts provide color-coded output:
- 🟢 Green: Test section headers
- 🟡 Yellow: Test progress
- 🔴 Red: Errors or failures

Each test will show:
1. The test being performed
2. The RADIUS packet being sent
3. The server's response
4. Test result

## Troubleshooting

If tests fail, check:
1. RADIUS server is running
2. Correct ports are open
3. Shared secret matches
4. User exists in the database
5. MongoDB is running (for accounting tests)

## Manual Testing

You can also test manually using radtest:

```bash
# Authentication test
radtest testuser testpass localhost 0 testing123

# Accounting test
radtest -t acct -i 1 -a testing123 localhost:1813 0 testing123
```

## Continuous Integration

These test scripts are integrated into the CI pipeline and run automatically on:
- Pull requests
- Merges to main branch
- Nightly builds

See [CI/CD Configuration](../.github/workflows/ci.yml) for details. 