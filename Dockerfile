FROM rust:1.65-bullseye AS build
WORKDIR /usr/src
COPY . /usr/src/
RUN cargo build --release

FROM scratch
COPY --from=build /etc/ssl/certs /etc/ssl/certs
COPY --from=build /usr/src/target/release/persona-insight /
COPY --from=build /lib/x86_64-linux-gnu /lib/x86_64-linux-gnu
COPY --from=build /lib64 /lib64
COPY --from=build /usr/lib/x86_64-linux-gnu/libcrypto.so* /usr/lib/
COPY --from=build /usr/lib/x86_64-linux-gnu/libdl.so* /usr/lib/
COPY --from=build /usr/lib/x86_64-linux-gnu/libm.so* /usr/lib/
COPY --from=build /usr/lib/x86_64-linux-gnu/libpthread.so* /usr/lib/
COPY --from=build /usr/lib/x86_64-linux-gnu/libssl.so* /usr/lib/
COPY --from=build /usr/lib/gcc/x86_64-linux-gnu/*/libgcc_s.so* /usr/lib/
COPY --from=build /usr/share/ca-certificates /usr/share/ca-certificates
ENTRYPOINT ["/persona-insight"]
