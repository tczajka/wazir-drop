#!/bin/bash

set -e
mkdir -p target

cg-bundler -o target/submission.rs
rustc +1.75 -O --edition 2021 target/submission.rs -o target/submission