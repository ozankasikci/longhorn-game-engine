use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::path::Path;

use crate::types::{PendingCommand, RemoteCommand, RemoteResponse};

const SOCKET_PATH: &str = "/tmp/longhorn-editor.sock";

/// Handle for the remote control server
pub struct RemoteServer {
    /// Receiver for incoming commands (polled by main loop)
    pub command_rx: Receiver<PendingCommand>,
    /// Thread handle
    _thread: thread::JoinHandle<()>,
}

impl RemoteServer {
    /// Start the remote control server in a background thread
    pub fn start() -> std::io::Result<Self> {
        // Remove stale socket file
        let socket_path = Path::new(SOCKET_PATH);
        if socket_path.exists() {
            std::fs::remove_file(socket_path)?;
        }

        // Create listener
        let listener = UnixListener::bind(SOCKET_PATH)?;
        listener.set_nonblocking(false)?;

        log::info!("Remote control server listening on {}", SOCKET_PATH);

        // Channel for commands
        let (command_tx, command_rx) = mpsc::channel();

        // Spawn server thread
        let thread = thread::spawn(move || {
            Self::server_loop(listener, command_tx);
        });

        Ok(Self {
            command_rx,
            _thread: thread,
        })
    }

    fn server_loop(listener: UnixListener, command_tx: Sender<PendingCommand>) {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    if let Err(e) = Self::handle_connection(stream, &command_tx) {
                        log::warn!("Connection error: {}", e);
                    }
                }
                Err(e) => {
                    log::error!("Accept error: {}", e);
                    break;
                }
            }
        }
    }

    fn handle_connection(
        stream: UnixStream,
        command_tx: &Sender<PendingCommand>,
    ) -> std::io::Result<()> {
        log::debug!("Remote client connected");

        let mut reader = BufReader::new(stream.try_clone()?);
        let mut writer = stream;

        loop {
            let mut line = String::new();
            let bytes_read = reader.read_line(&mut line)?;

            if bytes_read == 0 {
                log::debug!("Remote client disconnected");
                break;
            }

            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            log::debug!("Remote command: {}", line);

            // Parse command
            let response = match serde_json::from_str::<RemoteCommand>(line) {
                Ok(command) => {
                    // Create response channel
                    let (response_tx, response_rx) = mpsc::channel();

                    // Send command to main loop
                    if command_tx.send(PendingCommand { command, response_tx }).is_err() {
                        RemoteResponse::error("Editor shutting down")
                    } else {
                        // Wait for response
                        match response_rx.recv() {
                            Ok(resp) => resp,
                            Err(_) => RemoteResponse::error("No response from editor"),
                        }
                    }
                }
                Err(e) => RemoteResponse::error(format!("Invalid command: {}", e)),
            };

            // Send response
            let response_json = serde_json::to_string(&response).unwrap();
            log::debug!("Remote response: {}", response_json);
            writeln!(writer, "{}", response_json)?;
            writer.flush()?;
        }

        Ok(())
    }
}

impl Drop for RemoteServer {
    fn drop(&mut self) {
        // Clean up socket file
        let _ = std::fs::remove_file(SOCKET_PATH);
        log::info!("Remote control server stopped");
    }
}
