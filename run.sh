#!/bin/bash
export $(egrep -v '^#' .env | xargs)
echo "Running Revolt in detached mode."
docker run \
    -d \
    --name revolt \
    -p 8000:8000 \
    -p 9000:9000 \
    -e "DB_URI=$DB_URI" \
    -e "PUBLIC_URI=$PUBLIC_URI" \
    -e "PORTAL_URL=$PORTAL_URI" \
    revolt
