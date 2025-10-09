//! Client for sending commands to the editor control server

use crate::types::*;
use tokio::net::TcpStream;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use uuid::Uuid;

pub struct EditorControlClient {
    stream: Option<TcpStream>,
    port: u16,
}

impl EditorControlClient {
    pub fn new(port: u16) -> Self {
        Self {
            stream: None,
            port,
        }
    }

    pub async fn connect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let addr = format!("127.0.0.1:{}", self.port);
        let stream = TcpStream::connect(&addr).await?;
        self.stream = Some(stream);
        Ok(())
    }

    pub async fn send_command(&mut self, command: EditorCommand) -> Result<EditorResponse, Box<dyn std::error::Error>> {
        let stream = self.stream.as_mut().ok_or("Not connected")?;
        
        let message = EditorMessage {
            id: Uuid::new_v4().to_string(),
            command,
        };
        
        let request_json = serde_json::to_string(&message)?;
        stream.write_all(format!("{}\n", request_json).as_bytes()).await?;
        
        // Read response
        let (reader, _writer) = stream.split();
        let mut buf_reader = BufReader::new(reader);
        let mut line = String::new();
        buf_reader.read_line(&mut line).await?;
        
        let reply: EditorReply = serde_json::from_str(line.trim())?;
        Ok(reply.response)
    }

    pub async fn disconnect(&mut self) {
        if let Some(mut stream) = self.stream.take() {
            let _ = stream.shutdown().await;
        }
    }

    // Convenience methods for common operations
    
    pub async fn ping(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
        match self.send_command(EditorCommand::Ping).await? {
            EditorResponse::Pong => Ok(true),
            _ => Ok(false),
        }
    }

    pub async fn add_script(&mut self, entity_id: u32, script_path: String) -> Result<bool, Box<dyn std::error::Error>> {
        match self.send_command(EditorCommand::AddScript { entity_id, script_path }).await? {
            EditorResponse::Success => Ok(true),
            EditorResponse::Error { message } => Err(message.into()),
            EditorResponse::EntityNotFound { entity_id } => Err(format!("Entity {} not found", entity_id).into()),
            _ => Ok(false),
        }
    }

    pub async fn remove_script(&mut self, entity_id: u32, script_path: String) -> Result<bool, Box<dyn std::error::Error>> {
        match self.send_command(EditorCommand::RemoveScript { entity_id, script_path }).await? {
            EditorResponse::Success => Ok(true),
            EditorResponse::Error { message } => Err(message.into()),
            EditorResponse::EntityNotFound { entity_id } => Err(format!("Entity {} not found", entity_id).into()),
            EditorResponse::ScriptNotFound { script_path } => Err(format!("Script {} not found", script_path).into()),
            _ => Ok(false),
        }
    }

    pub async fn replace_script(&mut self, entity_id: u32, old_path: String, new_path: String) -> Result<bool, Box<dyn std::error::Error>> {
        match self.send_command(EditorCommand::ReplaceScript { entity_id, old_path, new_path }).await? {
            EditorResponse::Success => Ok(true),
            EditorResponse::Error { message } => Err(message.into()),
            _ => Ok(false),
        }
    }

    pub async fn get_scene_objects(&mut self) -> Result<Vec<SceneObject>, Box<dyn std::error::Error>> {
        match self.send_command(EditorCommand::GetSceneObjects).await? {
            EditorResponse::SceneObjects(objects) => Ok(objects),
            EditorResponse::Error { message } => Err(message.into()),
            _ => Err("Unexpected response".into()),
        }
    }

    pub async fn get_entity_scripts(&mut self, entity_id: u32) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        match self.send_command(EditorCommand::GetEntityScripts { entity_id }).await? {
            EditorResponse::EntityScripts(scripts) => Ok(scripts),
            EditorResponse::EntityNotFound { entity_id } => Err(format!("Entity {} not found", entity_id).into()),
            EditorResponse::Error { message } => Err(message.into()),
            _ => Err("Unexpected response".into()),
        }
    }

    pub async fn get_logs(&mut self, lines: Option<usize>) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        match self.send_command(EditorCommand::GetLogs { lines }).await? {
            EditorResponse::Logs(logs) => Ok(logs),
            EditorResponse::Error { message } => Err(message.into()),
            _ => Err("Unexpected response".into()),
        }
    }
}