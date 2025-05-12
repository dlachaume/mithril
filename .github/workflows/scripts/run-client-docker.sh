#!/bin/bash

set -e

if [[ $# -lt 1 ]]; then
  echo "Usage: $0 '<mithril-client command>'"
  exit 1
fi

CLIENT_CMD="$1"

# Run mithril-client and capture the output
OUTPUT=$(eval "$CLIENT_CMD")

# Search for the line containing 'docker run'
DOCKER_CMD=$(echo "$OUTPUT" | grep -E '^\s*docker run')

# Check if the Docker command was found
if [[ -n "$DOCKER_CMD" ]]; then
  echo "Extracted Docker command:"
  echo "$DOCKER_CMD"

  DOCKER_CMD_DETACHED="${DOCKER_CMD/docker run/docker run -d}"
  echo "Running Docker command in detached mode:"
  echo "$DOCKER_CMD_DETACHED"

  CONTAINER_ID=$(eval "$DOCKER_CMD_DETACHED")

  echo "Container started with ID: $CONTAINER_ID"
  echo "Waiting up to 15 seconds for 'Started opening Immutable DB'..."

  FOUND_LOG=false
  for ((i=1; i<=15; i++)); do
    if docker logs "$CONTAINER_ID" 2>&1 | grep -q "Started opening Immutable DB"; then
      FOUND_LOG=true
      break
    fi
    sleep 1
  done

  if [[ "$FOUND_LOG" == true ]]; then
    echo "✅ Found 'Started opening Immutable DB' in logs."
    exit 0
  else
    echo "❌ 'Started opening Immutable DB' not found within 15 seconds."
    docker logs "$CONTAINER_ID"
    exit 1
  fi

else
  echo "No Docker command found in mithril-client CLI command output."
  exit 1
fi
