FROM ubuntu:24.04

RUN DEBIAN_FRONTEND=noninteractive \
    apt-get update \
    && apt-get install -y ca-certificates \
    && update-ca-certificates

WORKDIR /spalhad

ENV RUST_BACKTRACE=full

COPY ./docker/launch-server.sh ./launch
COPY ./build/spalhad-server-bin ./server

EXPOSE 5000

CMD ["./launch"]
