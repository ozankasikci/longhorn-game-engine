//! TestHarness - manages editor process lifecycle for tests.

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::client::{EditorClient, DEFAULT_SOCKET_PATH};
use crate::error::EditorError;
use crate::responses::LogEntry;

/// Default timeout for waiting for the editor to start.
const STARTUP_TIMEOUT: Duration = Duration::from_secs(30);

/// Interval for checking if the socket is ready.
const POLL_INTERVAL: Duration = Duration::from_millis(100);

/// Test harness for managing editor process and client connection.
pub struct TestHarness {
    editor_process: Child,
    client: EditorClient,
    test_output_dir: PathBuf,
    socket_path: String,
}

impl TestHarness {
    /// Start the editor with the default test_project.
    pub fn start() -> Result<Self, EditorError> {
        Self::start_with_options(StartOptions::default())
    }

    /// Start the editor with a custom project path.
    pub fn start_with_project(project_path: impl AsRef<Path>) -> Result<Self, EditorError> {
        Self::start_with_options(StartOptions {
            project_path: Some(project_path.as_ref().to_path_buf()),
            ..Default::default()
        })
    }

    /// Start the editor with custom options.
    pub fn start_with_options(options: StartOptions) -> Result<Self, EditorError> {
        let socket_path = options.socket_path.clone().unwrap_or_else(|| DEFAULT_SOCKET_PATH.to_string());

        // Clean up any existing socket
        let _ = std::fs::remove_file(&socket_path);

        // Build the command
        let mut cmd = Command::new("cargo");
        cmd.args(["run", "--bin", "longhorn-editor"]);

        if let Some(project) = &options.project_path {
            cmd.args(["--", "--project", project.to_str().unwrap()]);
        }

        // Capture output for debugging
        cmd.stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Set working directory if specified
        if let Some(cwd) = &options.working_dir {
            cmd.current_dir(cwd);
        }

        let editor_process = cmd.spawn()
            .map_err(|e| EditorError::ProcessError(format!("Failed to spawn editor: {}", e)))?;

        // Wait for the socket to become available
        let start = Instant::now();
        let client = loop {
            if start.elapsed() > options.startup_timeout.unwrap_or(STARTUP_TIMEOUT) {
                return Err(EditorError::Timeout);
            }

            match EditorClient::connect(&socket_path) {
                Ok(mut client) => {
                    // Verify the connection works
                    if client.ping().is_ok() {
                        break client;
                    }
                }
                Err(_) => {
                    thread::sleep(POLL_INTERVAL);
                }
            }
        };

        // Set up test output directory
        let test_output_dir = options.test_output_dir.unwrap_or_else(|| {
            PathBuf::from("test_output")
        });
        std::fs::create_dir_all(&test_output_dir)?;

        Ok(Self {
            editor_process,
            client,
            test_output_dir,
            socket_path,
        })
    }

    /// Get a reference to the client.
    pub fn client(&mut self) -> &mut EditorClient {
        &mut self.client
    }

    /// Take a screenshot with automatic naming.
    ///
    /// Returns the path to the saved screenshot.
    pub fn screenshot(&mut self, step: &str) -> Result<PathBuf, EditorError> {
        let filename = format!("{}_{}.png", step, chrono_lite_timestamp());
        let path = self.test_output_dir.join(&filename);
        let path_str = path.to_string_lossy().to_string();

        self.client.take_screenshot(&path_str)?;
        Ok(path)
    }

    /// Read recent log entries.
    pub fn logs(&mut self, lines: usize) -> Result<Vec<LogEntry>, EditorError> {
        let result = self.client.get_log_tail(lines)?;
        Ok(result.entries)
    }

    /// Get the test output directory.
    pub fn test_output_dir(&self) -> &Path {
        &self.test_output_dir
    }

    /// Get the socket path being used.
    pub fn socket_path(&self) -> &str {
        &self.socket_path
    }

    /// Gracefully shutdown the editor.
    pub fn shutdown(mut self) -> Result<(), EditorError> {
        // Try to get the process to exit gracefully
        // For now, just kill it
        self.editor_process.kill()
            .map_err(|e| EditorError::ProcessError(format!("Failed to kill editor: {}", e)))?;
        self.editor_process.wait()
            .map_err(|e| EditorError::ProcessError(format!("Failed to wait for editor: {}", e)))?;

        // Clean up socket
        let _ = std::fs::remove_file(&self.socket_path);

        Ok(())
    }
}

impl Drop for TestHarness {
    fn drop(&mut self) {
        // Kill the editor process on drop
        let _ = self.editor_process.kill();
        let _ = self.editor_process.wait();

        // Clean up socket
        let _ = std::fs::remove_file(&self.socket_path);
    }
}

/// Options for starting the test harness.
#[derive(Debug, Clone, Default)]
pub struct StartOptions {
    /// Path to the project to load.
    pub project_path: Option<PathBuf>,

    /// Working directory for the cargo command.
    pub working_dir: Option<PathBuf>,

    /// Socket path to use.
    pub socket_path: Option<String>,

    /// Timeout for waiting for the editor to start.
    pub startup_timeout: Option<Duration>,

    /// Directory to store test output (screenshots, etc).
    pub test_output_dir: Option<PathBuf>,
}

/// Generate a simple timestamp for file naming (no external dependencies).
fn chrono_lite_timestamp() -> String {
    use std::time::SystemTime;
    let duration = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", duration.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn test_harness_start_and_ping() {
        let mut harness = TestHarness::start().expect("Failed to start harness");
        harness.client().ping().expect("Ping failed");
    }
}
