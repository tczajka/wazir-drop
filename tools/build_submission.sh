#!/bin/bash

set -e
mkdir -p target

cargo equip -o target/submission.rs --bin submission --no-rustfmt
rustc +1.75 -O --edition 2021 target/submission.rs -o target/submission