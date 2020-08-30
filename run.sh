#!/bin/bash
# Split at \n instead of space.
# https://unix.stackexchange.com/a/39482
set -f
IFS='
'

input=($(egrep -v '^#' .env))
prepended=(${input[@]/#/-e\"})
variables=${prepended[@]/%/\"}

unset IFS
set +f

echo "Running Revolt in detached mode."
docker run \
    -d \
    --name revolt \
    -p 8000:8000 \
    -p 9000:9000 \
    $variables \
    revolt
