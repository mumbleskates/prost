#!/usr/bin/env bash

set -euxo pipefail

for feature in std \
               detailed-errors \
               extended-diagnostics \
               arrayvec \
               bytestring \
               hashbrown \
               smallvec \
               thin-vec \
               tinyvec \
               self-copy-optimization \
               unroll-varint-encoding \
               ; do
  cargo clippy --workspace --all-targets --no-default-features --features $feature
  cargo test --workspace --all-targets --exclude fuzz --no-default-features --features $feature
done
