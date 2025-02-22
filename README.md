# Spalhad

From Portuguese "espalhado" (spread, distributed),
Spalhad is a distributed hash table WIP project.

This **does not** have fault tolerance... yet?

## Running Docker Example

First, build the image:
```sh
make build-server-image
```

Then put all four containers up:
```
docker compose up -d
```

Finally, run the client with the `client.sh` script:
```
./client.sh -b http://localhost:5500 put -k foo -v 123

./client.sh -b http://localhost:5500 get -k foo

./client.sh -b http://localhost:5501 put -k bar -v '"hello"'

./client.sh -b http://localhost:5501 get -k bar

./client.sh -b http://localhost:5502 put -k baz -v '[5, 6, 7, 8]'

./client.sh -b http://localhost:5502 get -k baz

./client.sh -b http://localhost:5503 put -k point -v '{ "x": 35, "y": -9 }'

./client.sh -b http://localhost:5503 get -k point
```

Note that e.g. the `put` command sent to `http://localhost:5503`
does not imply that the entry will be stored in that node,
instead, the storing node is chosen based on some maths.
The node to which the command was sent will communicate with the proper node.
Try querying to a different node:

```sh
./client.sh -b http://localhost:5501 get -k point
```
