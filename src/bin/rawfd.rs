// Let's find out what this file descriptor thing is about
use std::{fs::File, io::Read, os::unix::io::AsRawFd, path::Path};

use anyhow::Context;

fn main() -> anyhow::Result<()> {
    read_file_and_find_raw_fd("README.md")?;
    Ok(())
}

fn read_file_and_find_raw_fd(path: &str) -> anyhow::Result<()> {
    let path = Path::new(path);

    println!("Let's find the file at path {:?}", path);

    let mut file = File::open(path).context("Can't open file")?;

    let mut content = String::new();
    match file.read_to_string(&mut content) {
        Ok(nb_of_bytes) => println!("Reading this file works, it has {} bytes", nb_of_bytes),
        Err(e) => return Err(e).context("Can't read this file, sorry"),
    }
    let raw_file_descriptor = file.as_raw_fd();

    println!("Here is its raw file descriptor: {:?}", raw_file_descriptor);

    Ok(())
}
