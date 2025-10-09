//! TCP server for receiving editor control commands

use crate::{commands::EditorCommandHandler, types::*};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

pub struct EditorControlServer {
    handler: EditorCommandHandler,
    port: u16,
}

impl EditorControlServer {
    pub fn new(handler: EditorCommandHandler, port: u16) -> Self {
        Self { handler, port }
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let addr = format!("127.0.0.1:{}", self.port);
        let listener = TcpListener::bind(&addr).await?;
        
        log::info!("Editor control server listening on {}", addr);
        
        loop {
            let (socket, addr) = listener.accept().await?;
            log::info!("New connection from {}", addr);
            
            let handler = self.handler.clone();
            tokio::spawn(async move {
                if let Err(e) = handle_connection(socket, handler).await {
                    log::error!("Error handling connection: {}", e);
                }
            });
        }
    }
}

async fn handle_connection(
    mut socket: TcpStream,
    handler: EditorCommandHandler,
) -> Result<(), Box<dyn std::error::Error>> {
    let (reader, mut writer) = socket.split();
    let mut buf_reader = BufReader::new(reader);
    let mut line = String::new();
    
    loop {
        line.clear();
        let bytes_read = buf_reader.read_line(&mut line).await?;
        
        if bytes_read == 0 {
            // Connection closed
            break;
        }
        
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        
        // Parse the command
        let message: EditorMessage = match serde_json::from_str(line) {
            Ok(msg) => msg,
            Err(e) => {
                let error_response = EditorReply {
                    id: "unknown".to_string(),
                    response: EditorResponse::Error {
                        message: format!("Invalid JSON: {}", e),
                    },
                };
                let response_json = serde_json::to_string(&error_response)?;
                writer.write_all(format!("{}\n", response_json).as_bytes()).await?;
                continue;
            }
        };
        
        // Check for shutdown command before moving
        let is_shutdown = matches!(message.command, EditorCommand::Shutdown);
        
        // Execute the command
        let response = handler.execute_command(message.command);
        
        // Send the response
        let reply = EditorReply {
            id: message.id,
            response,
        };
        
        let response_json = serde_json::to_string(&reply)?;
        writer.write_all(format!("{}\n", response_json).as_bytes()).await?;
        
        // Handle shutdown command
        if is_shutdown && matches!(reply.response, EditorResponse::Success) {
            break;
        }
    }
    
    Ok(())
}

