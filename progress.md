This ought to become a blog post.

# Make a socket-based server-client

Instead of taming the channels head-on, let's try and get it to work using only sockets.

## Why do I get this error every time?

Even when setting socket permissions to `777`, I can't connect to it:

```
Error: Can not connect to socket

Caused by:
    Permission denied (os error 13)
```

It turns out the permissions had to be given in octal, so `0o600` (600 is plenty).

## Listen with `incoming()`

Listening on a socket works when doing this on the server side:

```rust
fn main() -> anyhow::Result<()> {
    let socket = SocketBuilder::new()
        .with_path(socket_path)?
        .with_permissions(0o700)?
        .build()?;

    for unix_stream in socket.listener.incoming() {
        match unix_stream {
            Ok(stream) => handle_stream(stream)?,
            Err(e) => {
                bail!(format!("{}", e));
            }
        }
    }
    Ok(())
}

fn handle_stream(mut stream: UnixStream) -> anyhow::Result<()> {
    println!("stream: {:?}", stream);
    let mut message = String::new();
    stream.read_to_string(&mut message)?;
    println!("{}", message);
    Ok(())
}
```

But this is blocking. If the client sends ten messages on the socket, the `read_to_string` function waits for them all.

## How to make the listener non-blocking?

I tried copying Sōzu's code but its asynchronous wrapping sounds like chinese to me.

Let's use the `set_nonblocking` method of `std::os::unix::net::UnixListener`.
Here is the documentation:

> This will result in the `accept` operation becoming nonblocking,
> i.e., immediately returning from their calls. If the IO operation is
> successful, `Ok` is returned and no further action is required. If the
> IO operation could not be completed and needs to be retried, an error
> with kind [`io::ErrorKind::WouldBlock`] is returned.

## Plotwist... don't

So I gave the SocketBuilder a nice `nonblocking(boolean)` method and…
I've got mixed feelings about this.
Calling the `accept()` method on a non-blocking listener yielded this error:

    Error: Failed at accepting a connection on the unix listener
    Caused by:
        Resource temporarily unavailable (os error 11)

That being said, I managed to make it all work with a blocking socket.

**Server side**:

```rust
// within main
    loop {
        let (unix_stream, socket_address) = socket
            .listener
            .accept()
            .context("Failed at accepting a connection on the unix listener")?;
        handle_stream(unix_stream)?;
    }

fn handle_stream(mut stream: UnixStream) -> anyhow::Result<()> {
    let mut message = String::new();
    stream
        .read_to_string(&mut message)
        .context("Failed at reading the unix stream")?;

    Ok(())
}
```

**client side**:

```rust
fn main() {

    // ...

    let unix_stream =
        UnixStream::connect(socket_path).context("Could not connect to unix socket")?;

    stream.write(request_as_bytes)?;
}
```

This prints the request on the server.

## Accept and deserialize a request

```rust
stream
    .read_to_string(&mut message)
    .context("Failed at reading the unix stream")?;

let request = serde_json::from_str::<CommandRequest>(&message)
    .context("could no deserialize request message")?;
```

This works.

## How not to send and receive the response

I'm trying this:

-   On the server side, reading the stream and, upon reception of a request, writing to it
-   On the client side, writing to the socket, and then listening to it

I get those errors:

**server side**

```
Parsed this request: CommandRequest { id: "My-urgent-request", version: 0, worker_id: None }

Error: Could not write response onto the unix stream

Caused by:
    Broken pipe (os error 32)
```

**client side**

```
This request has been written : "{\"id\":\"My-urgent-request\",\"version\":0}"

Error: Could not bind to the socket

Caused by:
    Address already in use (os error 98)
```

## The solution: don't create a listener on the client side, use the stream

The errors happened because I tried to create a listener on the client side, of wich there was no actual need.
What should be done:

-   On the server side,
    -   create the socket
    -   create a stream
    -   read the stream
    -   upon reception of a request, write to it
-   On the client side,
    -   create a stream,
    -   write to it,
    -   read from it

This works.

## Separate messages with new lines

By appending a newline to each response sent by the server:

```rust
response_as_string.push('\n');

let response_as_bytes = response_as_string.as_bytes();

stream
    .write(response_as_bytes)
    .context("Could not write response onto the unix stream")?;
```

We can read messages separately on the client.
To do that, the unix stream, that implements `Read` already, needs to be wrapped in a `std::io::BufReader`:

```rust
let mut buf_reader = BufReader::new(unix_stream);
```

And then the method `read_line()` reads the buffer until a `\n` is found.

```rust
// in a loop
let mut message = String::new();
    let _ = buf_reader
        .read_line(&mut message)
        .context("Failed at reading response line from the buffer");
```

It works pretty well.

Notice all this is done an blocking socket.
