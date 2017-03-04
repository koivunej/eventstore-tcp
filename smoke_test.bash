#!/bin/bash
#
# simple build all and test testclient against inmemory ES script

set -eu
set -o pipefail

readonly DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

cargo test --quiet

cd examples/testclient
cargo test --quiet
bash ./test_with_inmemory_es.bash
