#!/bin/sh
# Configure dependencies to build on Debian

set -eaux

# Set the target platform, if provided
if [ -n "${CROSS_DEB_ARCH:-}" ]; then
  if [ "$CROSS_DEB_ARCH" = "musl-linux-armhf" ]; then
    # Cross gives the wrong debian architecutre
    arch="armhf"

    # SFML can't find these for some reason. Set them globally
    # (this is a docker container!)
    echo "export SFML_INCLUDE_DIR=/usr/include" >> "/etc/profile.d/cfg-sfml.sh"
    echo "export SFML_LIBS_DIR=/usr/lib/arm-linux-gnueabihf" >> "/etc/profile.d/cfg-sfml.sh"
    chmod +x "/etc/profile.d/cfg-sfml.sh"
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
