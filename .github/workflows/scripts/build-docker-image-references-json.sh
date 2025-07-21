#!/usr/bin/env bash

set -e

echo "{"
first=1
for file in "$@"; do
  if [[ ! -f "$file" ]]; then
    echo "File not found: $file" >&2
    exit 1
  fi

  filename=$(basename "$file")
  component=${filename#docker-image-reference-}
  component=${component%.txt}
  if [ "$component" = "mithril-client-cli" ]; then
    component="mithril-client"
  fi

  image_ref=$(<"$file")

  if [ "$first" -eq 0 ]; then
    echo ","
  else
    first=0
  fi

  printf '  "%s": "%s"' "$component" "$image_ref"
done
echo
echo "}"
