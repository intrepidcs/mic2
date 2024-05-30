# libftdi is not available in the almalinux-release package
set -e

VERSION=1.5

echo "Downloading libftdi1-${VERSION}..."
curl -OL https://www.intra2net.com/en/developer/libftdi/download/libftdi1-${VERSION}.tar.bz2
tar -xf libftdi1-${VERSION}.tar.bz2
cd libftdi1-${VERSION}
echo "Building libftdi1-${VERSION}..."
cmake -S . -B build -DCMAKE_BUILD_TYPE=Release -DFTDI_EEPROM=OFF -Wno-dev
cmake --build build --target install --config Release
cd ..
echo "Done building libftdi1-${VERSION}!"