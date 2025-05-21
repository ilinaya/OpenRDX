#!/bin/bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
RADIUS_SERVER="localhost"
AUTH_PORT="1812"
ACCT_PORT="1813"
SECRET="openrdx"
USERNAME="testuser"
PASSWORD="testpass"
NAS_PORT="0"

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

# Function to test authentication
test_auth() {
    print_header "Testing Authentication"
    
    echo "Testing PAP authentication..."
    radtest -t pap $USERNAME $PASSWORD $RADIUS_SERVER:$AUTH_PORT $NAS_PORT $SECRET
    
    echo -e "\nTesting CHAP authentication..."
    radtest -t chap $USERNAME $PASSWORD $RADIUS_SERVER:$AUTH_PORT $NAS_PORT $SECRET
    
    echo -e "\nTesting MSCHAP authentication..."
    radtest -t mschap $USERNAME $PASSWORD $RADIUS_SERVER:$AUTH_PORT $NAS_PORT $SECRET
}

# Function to test accounting
test_accounting() {
    print_header "Testing Accounting"
    
    echo "Testing accounting start..."
    radtest -t accounting-start $USERNAME $PASSWORD $RADIUS_SERVER:$ACCT_PORT $NAS_PORT $SECRET
    
    echo -e "\nTesting accounting interim..."
    radtest -t accounting-interim $USERNAME $PASSWORD $RADIUS_SERVER:$ACCT_PORT $NAS_PORT $SECRET
    
    echo -e "\nTesting accounting stop..."
    radtest -t accounting-stop $USERNAME $PASSWORD $RADIUS_SERVER:$ACCT_PORT $NAS_PORT $SECRET
}

# Function to test invalid credentials
test_invalid_auth() {
    print_header "Testing Invalid Authentication"
    
    echo "Testing with invalid password..."
    radtest -t pap $USERNAME "wrongpass" $RADIUS_SERVER:$AUTH_PORT $NAS_PORT $SECRET
    
    echo -e "\nTesting with invalid username..."
    radtest -t pap "wronguser" $PASSWORD $RADIUS_SERVER:$AUTH_PORT $NAS_PORT $SECRET
}

# Function to test with different NAS ports
test_nas_ports() {
    print_header "Testing Different NAS Ports"
    
    echo "Testing with NAS-Port 1..."
    radtest -t pap $USERNAME $PASSWORD $RADIUS_SERVER:$AUTH_PORT 1 $SECRET
    
    echo -e "\nTesting with NAS-Port 2..."
    radtest -t pap $USERNAME $PASSWORD $RADIUS_SERVER:$AUTH_PORT 2 $SECRET
}

# Main execution
echo -e "${GREEN}Starting RADIUS Server Tests${NC}"

# Check if radtest is installed
check_radtest

# Run tests
test_auth
test_accounting
test_invalid_auth
test_nas_ports

echo -e "\n${GREEN}Tests completed${NC}" 