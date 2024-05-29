#!/bin/bash
set -e

PARENT_DIR=$(pwd)
SFML_VERSION="2.5.1"
SFML_URL="https://github.com/SFML/SFML/archive/refs/tags/${SFML_VERSION}.tar.gz"

mkdir -pv build_sfml && cd build_sfml
echo "Downloading SFML source ${SFML_VERSION}..."
curl -OL $SFML_URL
tar -xf ${SFML_VERSION}.tar.gz

echo "Building SFML ${SFML_VERSION}..."
cd SFML-${SFML_VERSION}
cmake -S . -B build -DCMAKE_BUILD_TYPE=Release -Wno-dev
cmake --build build --target install --config Release

echo "Setting up rust-sfml environment variables..."
export SFML_INCLUDE_DIR="$(pwd)/include"
export SFML_LIB_DIR="$(pwd)/build/lib"

cd $PARENT_DIR

echo "Done building SFML ${SFML_VERSION}!"