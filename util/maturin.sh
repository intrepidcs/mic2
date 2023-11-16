#!/bin/bash

py="/usr/bin/python3"
venv_dir=".venv"

echo "Verifying python3 exists...";
if [[ `test -f "$py"` -ne 0 ]]; then
    echo "Error: ${py} is not valid...";
    exit 1;
fi

echo "Creating virtual environment...";
$py -m venv $venv_dir
if [ $? -ne 0 ]; then
    echo "Error: Failed to create virtual environment at ${venv_dir}";
    exit 1;
fi

echo "Activating virtual environment at ${venv_dir}"
source $venv_dir/bin/activate
if [ $? -ne 0 ]; then
    echo "Error: Failed to source ${venv_dir}/bin/activate";
    exit 1;
fi

echo "Installing Maturin..."
pip install maturin
if [ $? -ne 0 ]; then
    echo "Error: Failed to pip modules.";
    exit 1;
fi

cd crates/python/
echo "Building/Installing crate..."
maturin develop
if [ $? -ne 0 ]; then
    echo "Error: Failed to maturin develop.";
    exit 1;
fi

echo "Running unit tests..."
python -m unittest discover
if [ $? -ne 0 ]; then
    echo "Error: Failed to run unit tests.";
    exit 1;
fi
echo "Finished."