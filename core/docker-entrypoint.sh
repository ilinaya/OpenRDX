#!/bin/bash

# Set default service type if not provided
SERVICE_TYPE=${SERVICE_TYPE:-"auth"}

# Set default ports based on service type
if [ "$SERVICE_TYPE" = "auth" ]; then
    export RADIUS_BIND_ADDR=${RADIUS_BIND_ADDR:-"0.0.0.0:1812"}
    echo "Starting RADIUS authentication service on $RADIUS_BIND_ADDR"
elif [ "$SERVICE_TYPE" = "acct" ]; then
    export RADIUS_BIND_ADDR=${RADIUS_BIND_ADDR:-"0.0.0.0:1813"}
    echo "Starting RADIUS accounting service on $RADIUS_BIND_ADDR"
else
    echo "Invalid service type: $SERVICE_TYPE"
    exit 1
fi

# Run the application with the appropriate service type
exec OpenRDX-Core