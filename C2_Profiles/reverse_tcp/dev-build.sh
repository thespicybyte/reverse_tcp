#!/usr/bin/env bash
# Fast dev rebuild — updates reverse_tcp:latest without re-pulling the full
# release image. Run this from the reverse_tcp/ directory.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

DOCKER_BUILDKIT=1 docker build \
    -f Dockerfile.dev \
    -t reverse_tcp:latest \
    -t ghcr.io/thespicybyte/reverse_tcp:v0.3.1 \
    .

INSTALL_DIR="/opt/Mythic/InstalledServices/reverse_tcp"
if [ -d "$INSTALL_DIR" ] && [ ! -L "$INSTALL_DIR" ]; then
    echo ""
    echo "Syncing source to $INSTALL_DIR..."
    rsync -a "$SCRIPT_DIR/" "$INSTALL_DIR/"
    # Point InstalledServices Dockerfile at the local dev image instead of the pinned release
    echo "FROM reverse_tcp:latest" > "$INSTALL_DIR/Dockerfile"
fi

echo ""
echo "Done. reverse_tcp:latest updated."
echo "Restart the container with: docker restart reverse_tcp"
