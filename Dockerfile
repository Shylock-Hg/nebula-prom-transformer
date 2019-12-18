FROM rustlang/rust:nightly as builder

ARG BUILD=/nebula-prom-transformer

ADD . ${BUILD}

RUN cd ${BUILD} && cargo build --release

FROM rustlang/rust:nightly

COPY --from=builder /nebula-prom-transformer/target/release/nebula-prom-transformer /
