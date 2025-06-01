#!/bin/bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
RADIUS_SERVER="localhost"
ACCT_PORT="1813"
SECRET="testing123"  # Using the default secret from our implementation
USERNAME="testuser"
PASSWORD="testpass"
NAS_PORT="0"
SESSION_ID="test-session-$(date +%s)"  # Generate unique session ID

# Function to print section headers
print_header() {
    echo -e "\n${YELLOW}=== $1 ===${NC}"
}

# Function to check if radtest is installed
check_radtest() {
    if ! command -v radtest &> /dev/null; then
        echo -e "${RED}Error: radtest is not installed${NC}"
        echo "Please install FreeRADIUS client tools:"
        echo "  Ubuntu/Debian: sudo apt-get install freeradius-utils"
        echo "  CentOS/RHEL: sudo yum install freeradius-utils"
        echo "  macOS: brew install freeradius"
        exit 1
    fi
}

# Function to test accounting start
test_accounting_start() {
    print_header "Testing Accounting Start"
    
    echo "Sending Accounting-Start packet..."
    radtest -t acct -i 1 -a $SECRET $RADIUS_SERVER:$ACCT_PORT $NAS_PORT $SECRET << EOF
User-Name = $USERNAME
Acct-Session-Id = $SESSION_ID
Acct-Status-Type = Start
Acct-Authentic = RADIUS
NAS-IP-Address = 127.0.0.1
NAS-Port = $NAS_PORT
Acct-Session-Time = 0
Acct-Input-Octets = 0
Acct-Output-Octets = 0
Acct-Input-Packets = 0
Acct-Output-Packets = 0
EOF
}

# Function to test accounting interim update
test_accounting_interim() {
    print_header "Testing Accounting Interim Update"
    
    echo "Sending Accounting-Interim-Update packet..."
    radtest -t acct -i 2 -a $SECRET $RADIUS_SERVER:$ACCT_PORT $NAS_PORT $SECRET << EOF
User-Name = $USERNAME
Acct-Session-Id = $SESSION_ID
Acct-Status-Type = Interim-Update
Acct-Authentic = RADIUS
NAS-IP-Address = 127.0.0.1
NAS-Port = $NAS_PORT
Acct-Session-Time = 300
Acct-Input-Octets = 1024
Acct-Output-Octets = 2048
Acct-Input-Packets = 10
Acct-Output-Packets = 20
EOF
}

# Function to test accounting stop
test_accounting_stop() {
    print_header "Testing Accounting Stop"
    
    echo "Sending Accounting-Stop packet..."
    radtest -t acct -i 3 -a $SECRET $RADIUS_SERVER:$ACCT_PORT $NAS_PORT $SECRET << EOF
User-Name = $USERNAME
Acct-Session-Id = $SESSION_ID
Acct-Status-Type = Stop
Acct-Authentic = RADIUS
NAS-IP-Address = 127.0.0.1
NAS-Port = $NAS_PORT
Acct-Session-Time = 600
Acct-Input-Octets = 2048
Acct-Output-Octets = 4096
Acct-Input-Packets = 20
Acct-Output-Packets = 40
Acct-Terminate-Cause = User-Request
EOF
}

# Function to test accounting on/off
test_accounting_on_off() {
    print_header "Testing Accounting On/Off"
    
    echo "Sending Accounting-On packet..."
    radtest -t acct -i 4 -a $SECRET $RADIUS_SERVER:$ACCT_PORT $NAS_PORT $SECRET << EOF
User-Name = $USERNAME
Acct-Session-Id = $SESSION_ID
Acct-Status-Type = Accounting-On
Acct-Authentic = RADIUS
NAS-IP-Address = 127.0.0.1
NAS-Port = $NAS_PORT
EOF

    echo -e "\nSending Accounting-Off packet..."
    radtest -t acct -i 5 -a $SECRET $RADIUS_SERVER:$ACCT_PORT $NAS_PORT $SECRET << EOF
User-Name = $USERNAME
Acct-Session-Id = $SESSION_ID
Acct-Status-Type = Accounting-Off
Acct-Authentic = RADIUS
NAS-IP-Address = 127.0.0.1
NAS-Port = $NAS_PORT
EOF
}

# Function to test invalid accounting packets
test_invalid_accounting() {
    print_header "Testing Invalid Accounting Packets"
    
    echo "Testing with invalid session ID..."
    radtest -t acct -i 6 -a $SECRET $RADIUS_SERVER:$ACCT_PORT $NAS_PORT $SECRET << EOF
User-Name = $USERNAME
Acct-Session-Id = invalid-session
Acct-Status-Type = Start
Acct-Authentic = RADIUS
NAS-IP-Address = 127.0.0.1
NAS-Port = $NAS_PORT
EOF

    echo -e "\nTesting with missing required attributes..."
    radtest -t acct -i 7 -a $SECRET $RADIUS_SERVER:$ACCT_PORT $NAS_PORT $SECRET << EOF
User-Name = $USERNAME
Acct-Status-Type = Start
EOF
}

# Function to test different NAS ports
test_nas_ports() {
    print_header "Testing Different NAS Ports"
    
    echo "Testing with NAS-Port 1..."
    radtest -t acct -i 8 -a $SECRET $RADIUS_SERVER:$ACCT_PORT 1 $SECRET << EOF
User-Name = $USERNAME
Acct-Session-Id = $SESSION_ID
Acct-Status-Type = Start
Acct-Authentic = RADIUS
NAS-IP-Address = 127.0.0.1
NAS-Port = 1
EOF

    echo -e "\nTesting with NAS-Port 2..."
    radtest -t acct -i 9 -a $SECRET $RADIUS_SERVER:$ACCT_PORT 2 $SECRET << EOF
User-Name = $USERNAME
Acct-Session-Id = $SESSION_ID
Acct-Status-Type = Start
Acct-Authentic = RADIUS
NAS-IP-Address = 127.0.0.1
NAS-Port = 2
EOF
}

# Main execution
echo -e "${GREEN}Starting RADIUS Accounting Server Tests${NC}"

# Check if radtest is installed
check_radtest

# Run tests
test_accounting_start
test_accounting_interim
test_accounting_stop
test_accounting_on_off
test_invalid_accounting
test_nas_ports

echo -e "\n${GREEN}Tests completed${NC}" 