#!/bin/bash
# Usage: ./scripts/bump-version.sh 0.3.0
set -e

VERSION="$1"
if [ -z "$VERSION" ]; then
  echo "Usage: $0 <version>"
  echo "Example: $0 0.3.0"
  exit 1
fi

# Update tauri.conf.json
sed -i "s/\"version\": \".*\"/\"version\": \"$VERSION\"/" src-tauri/tauri.conf.json

# Update Cargo.toml
sed -i "s/^version = \".*\"/version = \"$VERSION\"/" src-tauri/Cargo.toml

echo "Updated version to $VERSION in:"
echo "  - src-tauri/tauri.conf.json"
echo "  - src-tauri/Cargo.toml"
echo ""
echo "Next steps:"
echo "  git add -A && git commit -m \"v$VERSION\""
echo "  git tag v$VERSION && git push origin main --tags"
