#!/usr/bin/env sh

exec cargo run $RUST_MODE -p spalhad-client-bin -- "$@"
