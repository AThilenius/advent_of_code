#!/bin/bash
set -e

BASEDIR="$( cd "$(dirname "$0")" ; pwd -P )"

cd "${BASEDIR}/build"
cmake ..
make
./perflab "${BASEDIR}/blocks-small.bmp"
