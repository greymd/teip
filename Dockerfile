FROM rust:1.44.1-stretch AS builder

RUN apt-get update -yqq && apt-get install -y --no-install-recommends clang musl-tools ca-certificates
RUN CFLAGS="-fPIE" CC="musl-gcc -static" cargo install teip --features oniguruma

FROM ubuntu:20.04
COPY --from=builder /usr/local/cargo/bin/teip /usr/bin/

ENTRYPOINT ["/usr/bin/teip"]
CMD ["--help"]
