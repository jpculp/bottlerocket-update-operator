ARG ARCH
FROM bottlerocket-sdk-openssl-${ARCH} as build
ARG ARCH
USER root

ADD ./ /src/
WORKDIR /src/apiserver
RUN cargo install --locked --target ${ARCH}-bottlerocket-linux-musl --path . --root ./

FROM scratch
# Copy CA certificates store
COPY --from=build /etc/ssl /etc/ssl
COPY --from=build /etc/pki /etc/pki
# Copy binary
COPY --from=build /src/apiserver/bin/apiserver ./

EXPOSE 8080

ENTRYPOINT ["./apiserver"]