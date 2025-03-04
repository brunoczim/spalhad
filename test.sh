#!/bin/bash

set -e

assert_contains () {
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

if [ "$SKIP_BUILD" != 1 ]
then
    make build-server-image RUST_MODE=debug 
fi

docker compose down
docker compose up -d

log="get (name) on a node, expecting not found" \
    expected="Not found" \
    assert_contains ./client.sh -b http://localhost:5500 get -k name

log="get (name) on another node, expecting not found" \
    expected="Not found" \
    assert_contains ./client.sh -b http://localhost:5501 get -k name

log="put (name, luke) on a node, expecting new" \
    expected="new" \
    assert_contains ./client.sh -b http://localhost:5501 put -k name -v '"luke"'

log="get (name) on a node, expecting luke" \
    expected="\"luke\"" \
    assert_contains ./client.sh -b http://localhost:5502 get -k name

log="get (name) on another node, expecting luke" \
    expected="\"luke\"" \
    assert_contains ./client.sh -b http://localhost:5500 get -k name

log="get (point) on a node, expecting not found" \
    expected="Not found" \
    assert_contains ./client.sh -b http://localhost:5503 get -k point

log="get (point) on another node, expecting not found" \
    expected="Not found" \
    assert_contains ./client.sh -b http://localhost:5501 get -k point

log="put (name, john) on another node, expecting Update" \
    expected="Updated" \
    assert_contains ./client.sh -b http://localhost:5503 put -k name -v '"john"'

log="get (name) on a node, expecting john" \
    expected="\"john\"" \
    assert_contains ./client.sh -b http://localhost:5500 get -k name

log="get (name) on another node, expecting john" \
    expected="\"john\"" \
    assert_contains ./client.sh -b http://localhost:5503 get -k name

log="get (point) on a node, expecting not found" \
    expected="Not found" \
    assert_contains ./client.sh -b http://localhost:5501 get -k point

log="get (point) on another node, expecting not found" \
    expected="Not found" \
    assert_contains ./client.sh -b http://localhost:5502 get -k point

log="put (point, {x:93,y:-2}) on a node, expecting new" \
    expected="new" \
    assert_contains ./client.sh -b http://localhost:5503 put -k point -v \
        '{"x":93,"y":-2}'

log="get (point) on a node, expecting {x:93,y:-2}" \
    expected="\"x\":\\s*93" \
    assert_contains ./client.sh -b http://localhost:5501 get -k point

log="get (point) on another node, expecting {x:93,y:-2}" \
    expected="\"x\":\\s*93" \
    assert_contains ./client.sh -b http://localhost:5502 get -k point

log="get (name) on a node, expecting john" \
    expected="\"john\"" \
    assert_contains ./client.sh -b http://localhost:5500 get -k name

log="get (name) on another node, expecting john" \
    expected="\"john\"" \
    assert_contains ./client.sh -b http://localhost:5503 get -k name

log="get (balance) on a node, expecting not found" \
    expected="Not found" \
    assert_contains ./client.sh -b http://localhost:5502 get -k balance

log="get (balance) on another node, expecting not found" \
    expected="Not found" \
    assert_contains ./client.sh -b http://localhost:5501 get -k balance

log="put (point, {x:-451,y:3}) on a node, expecting Updated" \
    expected="Updated" \
    assert_contains ./client.sh -b http://localhost:5503 put -k point -v \
        '{"x":-451,"y":3}'

log="get (point) on a node, expecting {x:-451,y:3}" \
    expected="\"x\":\\s*-451" \
    assert_contains ./client.sh -b http://localhost:5501 get -k point

log="get (point) on another node, expecting {x:-451,y:3}" \
    expected="\"x\":\\s*-451" \
    assert_contains ./client.sh -b http://localhost:5502 get -k point

log="get (name) on a node, expecting john" \
    expected="\"john\"" \
    assert_contains ./client.sh -b http://localhost:5500 get -k name

log="get (name) on another node, expecting john" \
    expected="\"john\"" \
    assert_contains ./client.sh -b http://localhost:5503 get -k name

log="get (balance) on a node, expecting not found" \
    expected="Not found" \
    assert_contains ./client.sh -b http://localhost:5503 get -k balance

log="get (balance) on another node, expecting not found" \
    expected="Not found" \
    assert_contains ./client.sh -b http://localhost:5500 get -k balance

log="put (special, false) on a node, expecting new" \
    expected="new" \
    assert_contains ./client.sh -b http://localhost:5501 put -k special -v false

log="put (balance, 1300.25) on a node, expecting new" \
    expected="new" \
    assert_contains ./client.sh -b http://localhost:5502 put -k balance -v \
        1300.25

log="put (fib, [1,1,2,3,5,8,13,21]) on a node, expecting new" \
    expected="new" \
    assert_contains ./client.sh -b http://localhost:5503 put -k fib -v \
        [1,1,2,3,5,8,13,21]

log="put (magic, 42) on a node, expecting new" \
    expected="new" \
    assert_contains ./client.sh -b http://localhost:5500 put -k magic -v 42

log="put (ref, market) on a node, expecting new" \
    expected="new" \
    assert_contains ./client.sh -b http://localhost:5501 put -k ref -v \
        '"market"' 

log="put (magic, 139) on a node, expecting new" \
    expected="Updated" \
    assert_contains ./client.sh -b http://localhost:5502 put -k magic -v 139

log="get (name) on a node, expecting john" \
    expected="\"john\"" \
    assert_contains ./client.sh -b http://localhost:5503 get -k name

log="get (name) on another node, expecting john" \
    expected="\"john\"" \
    assert_contains ./client.sh -b http://localhost:5500 get -k name

log="get (name) on another ooother node, expecting john" \
    expected="\"john\"" \
    assert_contains ./client.sh -b http://localhost:5501 get -k name

log="get (ref) on a node, expecting market" \
    expected="\"market\"" \
    assert_contains ./client.sh -b http://localhost:5503 get -k ref

log="get (ref) on another node, expecting market" \
    expected="\"market\"" \
    assert_contains ./client.sh -b http://localhost:5500 get -k ref

log="get (ref) on another ooother node, expecting market" \
    expected="\"market\"" \
    assert_contains ./client.sh -b http://localhost:5501 get -k ref

log="get (magic) on a node, expecting market" \
    expected="139" \
    assert_contains ./client.sh -b http://localhost:5502 get -k magic

log="get (magic) on another node, expecting market" \
    expected="139" \
    assert_contains ./client.sh -b http://localhost:5503 get -k magic

log="get (magic) on another ooother node, expecting market" \
    expected="139" \
    assert_contains ./client.sh -b http://localhost:5501 get -k magic
