#!/bin/sh
# Configure dependencies to build on Debian

set -eaux

apt-get update
apt-get install -y \
  libsfml-dev \
  libudev-dev \
  libftdi1-dev
