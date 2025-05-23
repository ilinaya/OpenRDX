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
USERNAME="sopa"
PASSWORD="demacaco"
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

    echo -e "\nTesting MSCHAP authentication..."
    radtest -t mschap $USERNAME $PASSWORD $RADIUS_SERVER:$AUTH_PORT $NAS_PORT $SECRET
}

# Main execution
echo -e "${GREEN}Starting RADIUS Server Tests${NC}"

# Check if radtest is installed
check_radtest

# Run tests
test_auth

echo -e "\n${GREEN}Tests completed${NC}" 