#!/bin/bash

set -e

node_address_0=http://localhost:5500
node_address_1=http://localhost:5501
node_address_2=http://localhost:5502
node_address_3=http://localhost:5503

get_node_address () {
    local __var="node_address_$1"
    echo -n "${!__var}"
}

SETUP_NODES () {
    if [ "$SKIP_BUILD" != 1 ]
    then
        if [ -z "$RUST_MODE" ]
        then
            RUST_MODE=debug
        fi
        make build-server-image RUST_MODE="$RUST_MODE"
    fi

    docker compose down
    sleep 1
    docker container prune -f
    docker volume prune -a -f
    sleep 1
    docker compose up -d
    sleep 1

    echo
    echo "!!! nodes ready for $curr_suite !!!"
}

STOP_NODE () {
    docker compose stop ${SPALHAD_NODE_PREFIX:-spalhad-node}-"$1"
    sleep 1
}

START_NODE () {
    docker compose start ${SPALHAD_NODE_PREFIX:-spalhad-node}-"$1"
    sleep 1
}

SUITE () {
    if [ -n "$curr_suite" ]
    then
        echo
    fi
    curr_suite="$@"
    section_count=0
    echo "### $curr_suite ###"
    echo "#"
}

SECTION () {
    echo
    echo ">>> $@"
}

ASSERT_CONTAINS () {
    echo -n "$log... "
    set +e
    output="$("$@" 2>&1)"
    set -e
    if [ $? != 0 ]
    then
        echo >&2 $output
        exit 1
    fi

    if ! (echo "$output" | grep "$expected" > /dev/null)
    then
        echo >&2 "Incorrect output, found:"
        echo >&2 "$output"
        exit 1
    fi

    echo OK
}

ASSERT_GET () {
    node_address="$(get_node_address "$node")"
    log="get node=$node k=\"$key\" expected=($expected)" \
        ASSERT_CONTAINS ./client.sh -b "$node_address" get -k "$key"
}

ASSERT_PUT () {
    node_address="$(get_node_address "$node")"
    log="put node=$node k=\"$key\" v=$value expected=($expected)" \
        ASSERT_CONTAINS ./client.sh -b "$node_address" put -k "$key" -v "$value"
}
