#!/bin/sh
# Configure dependencies to build on Debian

set -eaux

# Set the target platform, if provided
if [ -n "${CROSS_DEB_ARCH:-}" ]; then
  if [ "$CROSS_DEB_ARCH" = "musl-linux-armhf" ]; then
    # Cross gives the wrong debian architecutre
    arch="armhf"

    if [ -n "$GITHUB_ENV" ]; then
      # SFML can't find these for some reason
      echo "SFML_INCLUDE_DIR=/usr/include/SFML" >> "$GITHUB_ENV"
      echo "SFML_LIBS_DIR=/usr/lib/arm-linux-gnueabihf" >> "$GITHUB_ENV"
    fi
  else
    arch="$CROSS_DEB_ARCH"
  fi
  
  dpkg --add-architecture "$arch"
  sfx=":${arch}"
fi

apt-get update
apt-get install -y \
  "libsfml-dev${sfx:-}" \
  "libudev-dev${sfx:-}" \
  "libftdi1-dev${sfx:-}"
