.PHONY: build sdk-openssl agent-image apiserver-image controller-image

UNAME_ARCH=$(shell uname -m)
ARCH ?= $(lastword $(subst :, ,$(filter $(UNAME_ARCH):%,x86_64:amd64 aarch64:arm64)))

image: build brupop-image

# Builds, Lints and Tests the Rust workspace
build:
	cargo fmt -- --check
	cargo build --locked
	cargo test --locked

brupop-image:
	docker build $(DOCKER_BUILD_FLAGS) \
		--build-arg ARCH="$(UNAME_ARCH)" \
		--tag "brupop-$(UNAME_ARCH)" \
		-f Dockerfile .