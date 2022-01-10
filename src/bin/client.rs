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

    // wrap the stream in a BufReader for easier reading, split at each zero byte
    let mut split_iterator = BufReader::new(unix_stream).split(0);

    while let Some(message) = split_iterator.next() {
        let message = message?;
        // parse it, it should be a JSON response
        let response = serde_json::from_slice::<Response>(&message)
            .context("could no deserialize request message")?;

        println!("The server responded: {:?}", response);

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
