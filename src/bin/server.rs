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
    println!("Hello, world!");

    let socket_path = "socket";

    let socket = SocketBuilder::new()
        .with_path(socket_path)
        .with_permissions(0o700)
        .nonblocking(false)
        .build()
        .context("Could not create the socket")?;

    println!("Starting server, Press Ctrl^C to stop...");

    loop {
        let (unix_stream, socket_address) = socket
            .listener
            .accept()
            .context("Failed at accepting a connection on the unix listener")?;
        println!(
            "Accepted connection. Stream: {:?}, address: {:?}",
            unix_stream, socket_address
        );
        handle_stream(unix_stream)?;
    }
}

fn handle_stream(mut stream: UnixStream) -> anyhow::Result<()> {
    // receive a message
    let mut message = String::new();
    stream
        .read_to_string(&mut message)
        .context("Failed at reading the unix stream")?;

    println!("{}", message);

    // parse it, it is a request after all
    let request = serde_json::from_str::<Request>(&message)
        .context("could no deserialize request message")?;

    println!("Parsed this request: {:?}", request);

    // create a response that matches the request
    let response: Response = match request.id.as_str() {
        "request" => Response::new("response", CommandStatus::Ok, "Roger that"),
        _ => Response::new("what", CommandStatus::Error, "Sorry what?"),
    };

    // send 10 processings before sending the final one
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

        println!("Sending response: {}", processing);
        stream
            .write(processing.as_bytes())
            .context("Could not write processing response onto the unix stream")?;

        sleep(Duration::from_secs(1));
    }

    let mut response_as_string = response
        .to_serialized_string()
        .context("Could not serialize response")?;

    response_as_string.push('\n');

    let response_as_bytes = response_as_string.as_bytes();

    stream
        .write(response_as_bytes)
        .context("Could not write response onto the unix stream")?;

    Ok(())
}
