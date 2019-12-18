# nebula-prom-transformer
Transform the nebula raw metrics data to prometheus.

# Usage

Run the Nebula Graph first, refer to https://github.com/vesoft-inc/nebula/blob/master/docs/manual-EN/3.build-develop-and-administration/1.build/1.build-source-code.md

## With Docker

```
./build-image.sh
docker run -d --net=host nebula-prom-transformer:latest /nebula-prom-transformer --nebula-port=11000
```

## Build from source

1. Install rust environment, refer to https://www.rust-lang.org/tools/install
2. `cargo build --release` then `cargo install --path .`
3. `nebula-prom-transformer --nebula-port=11000` or get help by `nebula-prom-transformer --help`

FAQ:
1. `nebula-prom-transformer --help`, `command not found`, then you need add the *~/.cargo/bin* to
`PATH`, i.e. `PATH="~/.cargo/bin:$PATH"`.

# TODO

1. Disable Rocket log color.
2. Flow control to limit QPS.
