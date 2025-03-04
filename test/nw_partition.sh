#!/bin/bash

set -e

. ./test/util.sh

SUITE Network Partition Tests

SETUP_NODES

SECTION populate

node=0 key=name value='"mark"' expected="new" ASSERT_PUT
node=1 key=special value=false expected="new" ASSERT_PUT
node=2 key=balance value=1300.25 expected="new" ASSERT_PUT
node=3 key=fib value='[1,1,2,3,5,8,13,21]' expected="new" ASSERT_PUT
node=0 key=magic value=42 expected="new" ASSERT_PUT
node=1 key=ref value='"hospital"' expected="new" ASSERT_PUT

SECTION tolerate one node down

STOP_NODE 0

node=1 key=name expected='"mark"' ASSERT_GET
node=2 key=name expected='"mark"' ASSERT_GET
node=3 key=name expected='"mark"' ASSERT_GET

node=1 key=magic value=101 expected="Updated" ASSERT_PUT
node=3 key=magic value=132 expected="Updated" ASSERT_PUT
node=2 key=magic value=125 expected="Updated" ASSERT_PUT

node=1 key=magic expected=125 ASSERT_GET
node=2 key=magic expected=125 ASSERT_GET
node=3 key=magic expected=125 ASSERT_GET

node=1 key=ref expected='"hospital"' ASSERT_GET
node=2 key=ref expected='"hospital"' ASSERT_GET
node=3 key=ref expected='"hospital"' ASSERT_GET

node=1 key=special expected=false ASSERT_GET
node=2 key=special expected=false ASSERT_GET
node=3 key=special expected=false ASSERT_GET

SECTION tolerate bad reads from recovered node

node=1 key=name expected='"mark"' ASSERT_GET
node=2 key=name expected='"mark"' ASSERT_GET
node=3 key=name expected='"mark"' ASSERT_GET

node=1 key=magic expected=125 ASSERT_GET
node=2 key=magic expected=125 ASSERT_GET
node=3 key=magic expected=125 ASSERT_GET

node=1 key=ref expected='"hospital"' ASSERT_GET
node=2 key=ref expected='"hospital"' ASSERT_GET
node=3 key=ref expected='"hospital"' ASSERT_GET

node=1 key=special expected=false ASSERT_GET
node=2 key=special expected=false ASSERT_GET
node=3 key=special expected=false ASSERT_GET
