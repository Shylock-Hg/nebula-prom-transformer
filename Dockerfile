#FROM rust:1.39.0 as builder
FROM rustlang/rust:nightly as builder

ARG BUILD=/nebula-prom-transformer

ADD . ${BUILD}

RUN cd ${BUILD} && cargo build --release

#FROM rust:1.39.0
#FROM alpine:3.10.3
FROM rustlang/rust:nightly

COPY --from=builder /nebula-prom-transformer/target/release/nebula-prom-transformer /
