RUST_MODE = release

SERVER_BIN_BUILD_ARGS_debug =
SERVER_BIN_BUILD_ARGS_release = --release
SERVER_BIN_BUILD_ARGS = $(SERVER_BIN_BUILD_ARGS_$(RUST_MODE))

build-server-bin:
	cargo build $(SERVER_BIN_BUILD_ARGS)

build-server-image: build-server-bin
	mkdir -p build
	cp target/$(RUST_MODE)/spalhad-server-bin build/
	DOCKER_BUILDKIT=1 docker build . -f Dockerfile \
					--tag local/spalhad/server:latest
