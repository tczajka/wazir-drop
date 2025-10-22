#!/bin/bash

set -e
mkdir -p target

rm -f target/submission.rs
cg-bundler --keep-docs -v -o target/submission.rs
rustc +1.75 -O --edition 2021 target/submission.rs -o target/submission
echo "Submission built successfully"
