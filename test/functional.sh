#!/bin/bash

set -e

. ./test/util.sh

SUITE Functional Tests

SETUP_NODES

SECTION fresh not found

node=0 key=name expected="Not found" ASSERT_GET
node=2 key=name expected="Not found" ASSERT_GET

SECTION first new key

node=1 key=name value='"luke"' expected="new" ASSERT_PUT

node=3 key=name expected='"luke"' ASSERT_GET
node=0 key=name expected='"luke"' ASSERT_GET

node=3 key=point expected="Not found" ASSERT_GET
node=2 key=point expected="Not found" ASSERT_GET

SECTION first key update

node=3 key=name value='"john"' expected="Updated" ASSERT_PUT

node=0 key=name expected='"john"' ASSERT_GET
node=2 key=name expected='"john"' ASSERT_GET

node=1 key=point expected="Not found" ASSERT_GET
node=2 key=point expected="Not found" ASSERT_GET

SECTION second new key

node=3 key=point value='{"x":93,"y":-2}' expected="new" ASSERT_PUT

node=1 key=point expected='"x":\s*93' ASSERT_GET
node=0 key=point expected='"x":\s*93' ASSERT_GET

node=3 key=name expected='"john"' ASSERT_GET
node=0 key=name expected='"john"' ASSERT_GET

node=2 key=balance expected="Not found" ASSERT_GET
node=1 key=balance expected="Not found" ASSERT_GET

SECTION second key update

node=3 key=point value='{"x":-451,"y":3}' expected="Updated" ASSERT_PUT

node=1 key=point expected='"x":\s*-451' ASSERT_GET
node=3 key=point expected='"x":\s*-451' ASSERT_GET

node=0 key=name expected='"john"' ASSERT_GET
node=2 key=name expected='"john"' ASSERT_GET

node=3 key=balance expected="Not found" ASSERT_GET
node=0 key=balance expected="Not found" ASSERT_GET

SECTION more keys behavior

node=1 key=special value=false expected="new" ASSERT_PUT
node=2 key=balance value=1300.25 expected="new" ASSERT_PUT
node=3 key=fib value='[1,1,2,3,5,8,13,21]' expected="new" ASSERT_PUT
node=0 key=magic value=42 expected="new" ASSERT_PUT
node=1 key=ref value='"market"' expected="new" ASSERT_PUT
node=2 key=magic value=139 expected="Updated" ASSERT_PUT

node=3 key=name expected='"john"' ASSERT_GET
node=0 key=name expected='"john"' ASSERT_GET
node=2 key=name expected='"john"' ASSERT_GET
node=1 key=name expected='"john"' ASSERT_GET

node=2 key=ref expected='"market"' ASSERT_GET
node=3 key=ref expected='"market"' ASSERT_GET
node=1 key=ref expected='"market"' ASSERT_GET
node=0 key=ref expected='"market"' ASSERT_GET

node=3 key=magic expected=139 ASSERT_GET
node=0 key=magic expected=139 ASSERT_GET
node=1 key=magic expected=139 ASSERT_GET
node=2 key=magic expected=139 ASSERT_GET

node=2 key=unknown expected="Not found" ASSERT_GET
node=0 key=unknown expected="Not found" ASSERT_GET
node=3 key=unknown expected="Not found" ASSERT_GET
node=1 key=unknown expected="Not found" ASSERT_GET
