#!/bin/bash
#
# simple build all and test testclient against inmemory ES script

set -eu
set -o pipefail

cargo test --quiet --all

cd testclient
bash ./test_with_inmemory_es.bash
