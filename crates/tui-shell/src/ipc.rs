//! Inter-process communication via Unix domain sockets.

use crate::error::{ShellError, ShellResult};
use crate::notification::Notification;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;

/// IPC message types.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum IpcMessage {
    // Shell -> App
    /// App received focus.
    Focus,
    /// App lost focus.
    Blur,
    /// Resize app area.
    Resize { width: u16, height: u16 },
    /// Request to save session state.
    SessionSave,
    /// Restore session state.
    SessionRestore { state: serde_json::Value },

    // App -> Shell
    /// Send notification.
    Notification(Notification),
    /// Request focus.
    RequestFocus,
    /// Share data with other apps.
    Data { key: String, value: serde_json::Value },
    /// Run a shell command.
    Command { name: String, args: Vec<String> },

    // Bidirectional
    /// Ping for connection check.
    Ping,
    /// Response to ping.
    Pong,

    // Responses
    /// Success response.
    Ok,
    /// Error response.
    Error { message: String },
}

/// IPC channel for communication.
pub struct IpcChannel {
    /// Unix socket stream.
    stream: UnixStream,
    /// Pending outgoing messages.
    pending_out: VecDeque<IpcMessage>,
    /// Pending incoming messages.
    pending_in: VecDeque<IpcMessage>,
    /// Read buffer.
    read_buffer: Vec<u8>,
}

impl IpcChannel {
    /// Connect to an existing socket.
    pub fn connect(path: &Path) -> ShellResult<Self> {
        let stream = UnixStream::connect(path).map_err(|e| ShellError::Ipc(e.to_string()))?;
        stream
            .set_nonblocking(true)
            .map_err(|e| ShellError::Ipc(e.to_string()))?;

        Ok(Self {
            stream,
            pending_out: VecDeque::new(),
            pending_in: VecDeque::new(),
            read_buffer: Vec::with_capacity(4096),
        })
    }

    /// Create from an existing stream.
    pub fn from_stream(stream: UnixStream) -> ShellResult<Self> {
        stream
            .set_nonblocking(true)
            .map_err(|e| ShellError::Ipc(e.to_string()))?;

        Ok(Self {
            stream,
            pending_out: VecDeque::new(),
            pending_in: VecDeque::new(),
            read_buffer: Vec::with_capacity(4096),
        })
    }

    /// Send a message.
    pub fn send(&mut self, msg: IpcMessage) -> ShellResult<()> {
        let json = serde_json::to_string(&msg)?;
        let data = format!("{}\n", json);

        self.stream
            .write_all(data.as_bytes())
            .map_err(|e| ShellError::Ipc(e.to_string()))?;

        Ok(())
    }

    /// Queue a message for sending.
    pub fn queue(&mut self, msg: IpcMessage) {
        self.pending_out.push_back(msg);
    }

    /// Flush pending outgoing messages.
    pub fn flush(&mut self) -> ShellResult<()> {
        while let Some(msg) = self.pending_out.pop_front() {
            self.send(msg)?;
        }
        Ok(())
    }

    /// Try to receive a message (non-blocking).
    pub fn recv(&mut self) -> ShellResult<Option<IpcMessage>> {
        // First check pending
        if let Some(msg) = self.pending_in.pop_front() {
            return Ok(Some(msg));
        }

        // Try to read from socket
        let mut buf = [0u8; 4096];
        match self.stream.read(&mut buf) {
            Ok(0) => Err(ShellError::Ipc("Connection closed".to_string())),
            Ok(n) => {
                self.read_buffer.extend_from_slice(&buf[..n]);
                self.parse_messages()
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(None),
            Err(e) => Err(ShellError::Ipc(e.to_string())),
        }
    }

    /// Receive a message (blocking).
    pub fn recv_blocking(&mut self) -> ShellResult<IpcMessage> {
        self.stream
            .set_nonblocking(false)
            .map_err(|e| ShellError::Ipc(e.to_string()))?;

        let mut buf = [0u8; 4096];
        loop {
            match self.stream.read(&mut buf) {
                Ok(0) => return Err(ShellError::Ipc("Connection closed".to_string())),
                Ok(n) => {
                    self.read_buffer.extend_from_slice(&buf[..n]);
                    if let Some(msg) = self.parse_messages()? {
                        self.stream
                            .set_nonblocking(true)
                            .map_err(|e| ShellError::Ipc(e.to_string()))?;
                        return Ok(msg);
                    }
                }
                Err(e) => return Err(ShellError::Ipc(e.to_string())),
            }
        }
    }

    /// Parse messages from read buffer.
    fn parse_messages(&mut self) -> ShellResult<Option<IpcMessage>> {
        // Find newline
        if let Some(pos) = self.read_buffer.iter().position(|&b| b == b'\n') {
            let line: Vec<u8> = self.read_buffer.drain(..=pos).collect();
            let json = String::from_utf8_lossy(&line[..line.len() - 1]);
            let msg: IpcMessage = serde_json::from_str(&json)?;
            return Ok(Some(msg));
        }
        Ok(None)
    }

    /// Check if connected.
    pub fn is_connected(&self) -> bool {
        // Try a zero-byte write to check connection
        self.stream.peer_addr().is_ok()
    }

    /// Send ping and wait for pong.
    pub fn ping(&mut self) -> ShellResult<bool> {
        self.send(IpcMessage::Ping)?;

        // Wait for pong with timeout
        self.stream
            .set_read_timeout(Some(std::time::Duration::from_secs(1)))
            .map_err(|e| ShellError::Ipc(e.to_string()))?;

        match self.recv_blocking() {
            Ok(IpcMessage::Pong) => Ok(true),
            Ok(_) => Ok(false),
            Err(_) => Ok(false),
        }
    }
}

/// Create a socket path for an app.
pub fn socket_path(app_id: u64) -> std::path::PathBuf {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
        .unwrap_or_else(|_| "/tmp".to_string());

    std::path::PathBuf::from(runtime_dir)
        .join("tui-shell")
        .join(format!("app-{}.sock", app_id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_serialization() {
        let msg = IpcMessage::Notification(Notification::info("test", "Hello"));
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("notification"));

        let restored: IpcMessage = serde_json::from_str(&json).unwrap();
        assert!(matches!(restored, IpcMessage::Notification(_)));
    }

    #[test]
    fn test_socket_path() {
        let path = socket_path(123);
        assert!(path.to_string_lossy().contains("app-123.sock"));
    }
}
