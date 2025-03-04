#!/usr/bin/env sh

RUST_FLAGS=

if [ "$RUST_MODE" = release ]
then
    RUST_FLAGS=--release
fi

exec cargo run $RUST_FLAGS -p spalhad-client-bin -- "$@"
