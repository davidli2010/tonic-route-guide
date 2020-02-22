# tonic_route_guide

A gRPC example using [tonic](https://github.com/hyperium/tonic), comes from [gRPC tutorials](https://grpc.io/docs/tutorials/basic/java/) 
and reference [tonic example](https://github.com/hyperium/tonic/blob/master/examples/routeguide-tutorial.md#tonic-build).

[Here](https://github.com/davidli2010/grpc-route-guide) is another implementation of this example using [grpc-rs](https://github.com/tikv/grpc-rs).

## Run

You can inspect this example by compiling and running the example server in one shell session:
```Bash
cargo run --bin server
    Finished dev [unoptimized + debuginfo] target(s) in 0.18s
     Running `target/debug/server`
listening on 127.0.0.1:8980
```

And then running the client in another:
```Bash
cargo run --bin client
    Finished dev [unoptimized + debuginfo] target(s) in 0.13s
     Running `target/debug/client 59519`
...
```
