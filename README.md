# ip_filter

Trying out ip filtering by intercepting the connect syscall.

## Build


### MacOS

```sh
cargo build --release && \
    export DYLD_INSERT_LIBRARIES="target/release/libnetwork_filter.dylib" && \
    export DYLD_FORCE_FLAT_NAMESPACE=1 && \
    export BLOCKED_IPS="192.168.1.1,8.8.8.8" && \
    cargo run --release
```

### Linux

```sh
cargo build --release && \
    export LD_PRELOAD="target/release/libnetwork_filter.so" && \
    export BLOCKED_IPS="192.168.1.1,8.8.8.8" && \
    cargo run --release
```
