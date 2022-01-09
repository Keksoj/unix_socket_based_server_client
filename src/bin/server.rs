// this is a server
// create a unix_listener that accepts connections from client
use std::{
    io::{Read, Write},
    os::unix::net::UnixStream,
    thread::sleep,
    time::Duration,
};

use anyhow::Context;

use unix_socket_based_client_server::{
    message::{CommandStatus, Request, Response},
    socket::SocketBuilder,
};

fn main() -> anyhow::Result<()> {
    let socket_path = "socket";

    // Create the socket
    // Have a look at socket.rs to see what this does
    let socket = SocketBuilder::new()
        .with_path(socket_path)
        .with_permissions(0o700)
        .nonblocking(false)
        .build()
        .context("Could not create the socket")?;

    println!("Starting the unix socket server, Press Ctrl^C to stop...");

    // the loop allows to handle several connections, one after the other
    loop {
        // accept_connection() is a wrapper around UnixListener::accept(), check socket.rs
        let (unix_stream, socket_address) = socket.accept_connection()?;

        println!(
            "Accepted connection. Stream: {:?}, address: {:?}",
            unix_stream, socket_address
        );

        handle_connection(unix_stream)?;
    }
}

fn handle_connection(mut stream: UnixStream) -> anyhow::Result<()> {
    // receive a message using normal read logic
    let mut message = String::new();
    stream
        .read_to_string(&mut message)
        .context("Failed at reading the unix stream")?;

    println!("{}", message);

    // parse it, it should be a JSON request
    let request = serde_json::from_str::<Request>(&message)
        .context("could no deserialize request message")?;

    println!("Parsed this request: {:?}", request);

    // Emulate processing time
    // send 10 processings responses every second before sending the final one
    for _ in 0..9 {
        let mut processing = Response::new(
            "processing",
            CommandStatus::Processing,
            "still processing...",
        )
        .to_serialized_string()
        .context("Could not serialize response")?;

        // add a newline, to separate instructions
        processing.push('\n');

        // pretty normal write logic
        println!("Sending response: {}", processing);
        stream
            .write(processing.as_bytes())
            .context("Could not write processing response onto the unix stream")?;

        sleep(Duration::from_secs(1));
    }

    // create a response that matches the request
    let response: Response = match request.id.as_str() {
        "request" => Response::new("response", CommandStatus::Ok, "Roger that"),
        _ => Response::new("what", CommandStatus::Error, "Sorry what?"),
    };

    let mut response_as_string = response
        .to_serialized_string()
        .context("Could not serialize response")?;

    // the newline is a separator, so that the client can distinguish between responses
    response_as_string.push('\n');

    // the usual write logic
    stream
        .write(response_as_string.as_bytes())
        .context("Could not write response onto the unix stream")?;

    Ok(())
}
