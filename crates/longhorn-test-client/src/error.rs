use thiserror::Error;

/// Errors that can occur when communicating with the editor.
#[derive(Debug, Error)]
pub enum EditorError {
    /// Failed to connect to the editor socket.
    #[error("Failed to connect to editor: {0}")]
    ConnectionFailed(#[from] std::io::Error),

    /// Failed to serialize a command.
    #[error("Failed to serialize command: {0}")]
    SerializeFailed(#[source] serde_json::Error),

    /// Failed to deserialize a response.
    #[error("Failed to deserialize response: {0}")]
    DeserializeFailed(#[source] serde_json::Error),

    /// The editor returned an error response.
    #[error("Editor error: {0}")]
    EditorError(String),

    /// The response was missing expected data.
    #[error("Response missing expected data")]
    MissingData,

    /// Unexpected response type.
    #[error("Unexpected response type")]
    UnexpectedResponse,

    /// Timeout waiting for response.
    #[error("Timeout waiting for response")]
    Timeout,

    /// Editor process failed to start or crashed.
    #[error("Editor process error: {0}")]
    ProcessError(String),

    /// Test harness error.
    #[error("Test harness error: {0}")]
    HarnessError(String),
}
