use std::{
    fs::{metadata, remove_file, set_permissions, Permissions},
    os::unix::{
        fs::PermissionsExt,
        io::{AsRawFd, RawFd},
        net::{SocketAddr, UnixListener, UnixStream},
    },
};

use anyhow::{bail, Context};

#[derive(Debug)]
pub struct Socket {
    pub path: String,
    pub listener: UnixListener,
    pub nonblocking: bool,
    pub permissions: Option<Permissions>,
}

impl Socket {
    pub fn get_raw_fd(&self) -> RawFd {
        self.listener.as_raw_fd()
    }

    pub fn set_nonblocking(&mut self, nonblocking: bool) -> anyhow::Result<()> {
        self.listener
            .set_nonblocking(nonblocking)
            .context(match nonblocking {
                true => "Could not set this socket to be nonblocking",
                false => "Could not set this socket to be blocking",
            })
    }

    pub fn accept_connection(&self) -> anyhow::Result<(UnixStream, SocketAddr)> {
        self.listener.accept().context(format!(
            "Could not accept connection on socket {}",
            self.path
        ))
    }
}

pub struct SocketBuilder {
    path: Option<String>,
    listener: Option<UnixListener>,
    nonblocking: Option<bool>,
    permissions: Option<Permissions>,
}

impl SocketBuilder {
    pub fn new() -> Self {
        Self {
            path: None,
            listener: None,
            nonblocking: None,
            permissions: None,
        }
    }

    pub fn with_path<T>(self, path: T) -> Self
    where
        T: ToString,
    {
        Self {
            path: Some(path.to_string()),
            listener: self.listener,
            nonblocking: self.nonblocking,
            permissions: self.permissions,
        }
    }

    pub fn nonblocking(self, nonblocking: bool) -> Self {
        Self {
            path: self.path,
            listener: self.listener,
            nonblocking: Some(nonblocking),
            permissions: self.permissions,
        }
    }

    /// ex: "0o600"
    pub fn with_permissions(self, permissions: u32) -> Self {
        let permissions = Permissions::from_mode(permissions);

        println!("Permissions are set.");

        Self {
            path: self.path,
            listener: self.listener,
            nonblocking: self.nonblocking,
            permissions: Some(permissions),
        }
    }

    pub fn build(self) -> anyhow::Result<Socket> {
        println!("Creating socket...");
        if self.path.is_none() {
            bail!("Please provide a path first");
        }

        let cloned_path = self.path.clone().unwrap();
        let addr = self.path.unwrap();

        println!(
            "Checking for the presence of a unix socket at path `{}`",
            addr
        );

        if metadata(&addr).is_ok() {
            println!("A socket is already present. Deleting...");
            remove_file(&addr)
                .with_context(|| format!("could not delete previous socket at {:?}", addr))?;
        }

        let unix_listener = UnixListener::bind(&addr).context("could not create unix socket")?;

        if self.permissions.is_some() {
            set_permissions(&addr, self.permissions.clone().unwrap())
                .context("could not set the unix socket permissions.")?;
        } else {
            println!("Warning, no permissions set.")
        }

        // if no value is set, the socket defaults to blocking
        let nonblocking = match self.nonblocking {
            Some(bool_value) => bool_value,
            None => false,
        };

        unix_listener
            .set_nonblocking(nonblocking)
            .context("Could not set unix listener to blocking or unblocking")?;

        let socket = Socket {
            path: cloned_path,
            listener: unix_listener,
            nonblocking,
            permissions: self.permissions,
        };

        println!("Successfully created socket: {:#?}", socket);

        Ok(socket)
    }
}
