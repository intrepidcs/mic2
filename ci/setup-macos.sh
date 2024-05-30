#!/bin/sh

set -eaux

brew install libftdi sfml

if [ -n "$GITHUB_ENV" ]; then
  # Configure environment for future steps
  echo "SFML_INCLUDE_DIR=$(brew --prefix)/opt/sfml/include" >> "$GITHUB_ENV"
  echo "SFML_LIBS_DIR=$(brew --prefix)/lib" >> "$GITHUB_ENV"
fi
