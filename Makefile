.PHONY: build sdk-openssl apiserver-image

UNAME_ARCH=$(shell uname -m)
ARCH ?= $(lastword $(subst :, ,$(filter $(UNAME_ARCH):%,x86_64:amd64 aarch64:arm64)))

images: apiserver-image

# Builds, Lints and Tests the Rust workspace
build:
	cargo fmt -- --check
	cargo build --locked
	cargo test --locked

# Augment the bottlerocket-sdk image with openssl built with the musl toolchain
sdk-openssl:
	docker build $(DOCKER_BUILD_FLAGS) \
		--build-arg ARCH="$(UNAME_ARCH)" \
		--tag "bottlerocket-sdk-openssl-$(UNAME_ARCH)" \
		-f Dockerfile.sdk_openssl .

apiserver-image: sdk-openssl
	docker build $(DOCKER_BUILD_FLAGS) \
		--build-arg ARCH="$(UNAME_ARCH)" \
		--tag "brupop-apiserver" \
		-f apiserver/Dockerfile .