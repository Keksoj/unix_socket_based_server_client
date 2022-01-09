// create a UnixStream that connects to the socket, write on it, read from it
use std::{
    io::{BufRead, BufReader, Write},
    os::unix::net::UnixStream,
};

use anyhow::Context;

use unix_socket_based_client_server::message::{CommandStatus, Request, Response};

fn main() -> anyhow::Result<()> {
    println!("Starting the unix socket client...");

    let socket_path = "socket";

    // Connect to the socket
    let mut unix_stream =
        UnixStream::connect(socket_path).context("Could not connect to unix socket")?;

    write_request_on_the_socket(&mut unix_stream)
        .context("Could not write a request on the socket")?;

    // wrap the stream in a BufReader for easier reading
    let mut buf_reader = BufReader::new(unix_stream);

    // the loop is here so we can receive several responses from the server
    loop {
        let response = receive_incoming_message(&mut buf_reader)
            .context("Could not receive incoming message")?;

        // Depending on the response status, keep listening or exit the loop
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

fn write_request_on_the_socket(unix_stream: &mut UnixStream) -> anyhow::Result<()> {
    // Create a request, serialize it and convert to bytes
    let request = Request::new("request", "This is a request, please respond");
    let request_as_string = request
        .to_serialized_string()
        .context("failed at serializing request")?;
    let request_as_bytes = request_as_string.as_bytes();

    // write the request on the stream
    unix_stream
        .write(request_as_bytes)
        .context("Writing bytes failed")?;

    // this is necessary se we can read from the stream afterwards
    unix_stream
        .shutdown(std::net::Shutdown::Write)
        .context("Could not shut down Write on the stream")?;

    println!("This request has been written : {:?}", request_as_string);

    Ok(())
}

fn receive_incoming_message(buf_reader: &mut BufReader<UnixStream>) -> anyhow::Result<Response> {
    let mut message = String::new();

    // the read_line() is a BufReader method to stop reading at a newline
    // because we now the server separates all messages with a newline
    let _ = buf_reader
        .read_line(&mut message)
        .context("Failed at reading response line from the buffer");

    // parse it, it should be a JSON response
    let response = serde_json::from_str::<Response>(&message)
        .context("could no deserialize request message")?;

    println!("The server responded: {:?}", response);

    Ok(response)
}
