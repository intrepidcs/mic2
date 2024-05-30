#!/bin/sh
# Configure dependencies to build on Debian

set -eaux

if [ -n "$CROSS_DEB_ARCH" ]; then
  dpkg --add-architecture "$CROSS_DEB_ARCH"
  sfx=":${CROSS_DEB_ARCH}"
fi

apt-get update
apt-get install -y \
  "libsfml-dev${sfx:-}" \
  "libudev-dev${sfx:-}" \
  "libftdi1-dev${sfx:-}"
