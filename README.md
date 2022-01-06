# Unix-socket based client/server

In order to [dig into Sōzu channels](https://github.com/Keksoj/stream_stuff_on_a_sozu_channel),
I had to dig into the workings of unix sockets.

## What this repo contains

-   a small socket library to create a socket, `src/socket.rs`
-   a simple socket-based server that creates a socket, uses its listener, writes responses back
-   a simple socket-based client that connects to the socket, writes to the stream and read from it.

## How to run

Please [install Rust](https://www.rust-lang.org/tools/install), it is awesome,
then clone this repo and do:

    cargo run --bin server

And then, **in a separate terminal**:

    cargo run --bin client
