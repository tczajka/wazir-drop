#!/bin/bash

set -e
mkdir -p target

rm -f target/submission.rs
cg-bundler --keep-docs -v -o target/submission.rs
rustc +1.75 -O --edition 2021 -C target-feature=+popcnt,+sse,+sse2,+sse3,+ssse3,+sse4.1,+sse4.2,+avx target/submission.rs -o target/submission
echo "Submission built successfully"
