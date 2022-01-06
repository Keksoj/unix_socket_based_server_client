// create a UnixStream that connects to the socket, write on it, read from it
use std::{
    io::{BufRead, BufReader, Write},
    os::unix::net::UnixStream,
};

use anyhow::Context;

use unix_socket_based_client_server::message::{CommandStatus, Request, Response};

fn main() -> anyhow::Result<()> {
    let socket_path = "socket";

    // Connect to the socket
    let mut unix_stream =
        UnixStream::connect(socket_path).context("Could not connect to unix socket")?;

    write_request_onto_stream(&mut unix_stream)
        .context("Could not write request onto the unix stream")?;

    let mut buf_reader = BufReader::new(unix_stream);

    loop {
        // receive a message
        let mut message = String::new();
        let _ = buf_reader
            .read_line(&mut message)
            .context("Failed at reading response line from the buffer");

        println!("Received a line:");
        // parse it, it is a request after all
        let response = serde_json::from_str::<Response>(&message)
            .context("could no deserialize request message")?;
        println!("{:?}", response);

        match response.status {
            CommandStatus::Ok => {
                println!("We are done here!");
                break;
            }
            CommandStatus::Error => {
                println!("The server finally replied with an error");
                break;
            }
            CommandStatus::Processing => {
                continue;
            }
        }
    }

    Ok(())
}

fn write_request_onto_stream(stream: &mut UnixStream) -> anyhow::Result<()> {
    let request = Request::new("request", "This is a request, please respond");

    let request_as_string = request
        .to_serialized_string()
        .context("failed at serializing request")?;

    let request_as_bytes = request_as_string.as_bytes();

    stream
        .write(request_as_bytes)
        .context("Writing bytes failed")?;

    // stream
    // .flush()
    // .context("Could not flush the stream after write ")?;

    stream.shutdown(std::net::Shutdown::Write)?;
    println!("This request has been written : {:?}", request_as_string);

    Ok(())
}
