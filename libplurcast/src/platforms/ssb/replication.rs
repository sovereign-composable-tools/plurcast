//! SSB replication and pub server connections
//!
//! This module handles connections to SSB pub servers and the replication protocol.

use super::keypair::SSBKeypair;
use crate::error::{PlatformError, Result};

/// Parsed multiserver address for SSB pub servers
#[derive(Debug, Clone, PartialEq)]
pub struct PubAddress {
    /// Protocol (always "net" for TCP/IP)
    pub protocol: String,

    /// Hostname or IP address
    pub host: String,

    /// TCP port number
    pub port: u16,

    /// Authentication method (always "shs" for Secret Handshake)
    pub auth: String,

    /// Base64-encoded Ed25519 public key
    pub pubkey: String,
}

impl PubAddress {
    /// Parse a multiserver address string
    pub fn parse(address: &str) -> Result<Self> {
        let parts: Vec<&str> = address.split('~').collect();
        if parts.len() != 2 {
            return Err(PlatformError::Validation(format!(
                "Invalid multiserver address format: expected 'net:host:port~shs:key', got '{}'",
                address
            ))
            .into());
        }

        let connection_part = parts[0];
        let auth_part = parts[1];

        let conn_parts: Vec<&str> = connection_part.split(':').collect();
        if conn_parts.len() != 3 {
            return Err(PlatformError::Validation(format!(
                "Invalid connection format: expected 'net:host:port', got '{}'",
                connection_part
            ))
            .into());
        }

        let protocol = conn_parts[0];
        let host = conn_parts[1];
        let port_str = conn_parts[2];

        if protocol != "net" {
            return Err(PlatformError::Validation(format!(
                "Unsupported protocol '{}': only 'net' (TCP/IP) is supported",
                protocol
            ))
            .into());
        }

        if host.is_empty() {
            return Err(PlatformError::Validation("Host cannot be empty".to_string()).into());
        }

        let port = port_str.parse::<u16>().map_err(|_| {
            PlatformError::Validation(format!(
                "Invalid port number '{}': must be between 1 and 65535",
                port_str
            ))
        })?;

        let auth_parts: Vec<&str> = auth_part.split(':').collect();
        if auth_parts.len() != 2 {
            return Err(PlatformError::Validation(format!(
                "Invalid auth format: expected 'shs:key', got '{}'",
                auth_part
            ))
            .into());
        }

        let auth = auth_parts[0];
        let pubkey = auth_parts[1];

        if auth != "shs" {
            return Err(PlatformError::Validation(format!(
                "Unsupported auth method '{}': only 'shs' (Secret Handshake) is supported",
                auth
            ))
            .into());
        }

        if pubkey.is_empty() {
            return Err(PlatformError::Validation("Public key cannot be empty".to_string()).into());
        }

        use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
        BASE64.decode(pubkey).map_err(|e| {
            PlatformError::Validation(format!("Invalid base64 in public key: {}", e))
        })?;

        Ok(Self {
            protocol: protocol.to_string(),
            host: host.to_string(),
            port,
            auth: auth.to_string(),
            pubkey: pubkey.to_string(),
        })
    }

    /// Format the pub address back to multiserver format
    pub fn to_string(&self) -> String {
        format!(
            "{}:{}:{}~{}:{}",
            self.protocol, self.host, self.port, self.auth, self.pubkey
        )
    }

    /// Get the socket address for TCP connection
    pub fn socket_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

/// Connection to an SSB pub server
#[derive(Debug)]
pub struct PubConnection {
    /// Pub server address
    pub address: PubAddress,

    /// Connection status
    pub connected: bool,

    /// Last connection attempt timestamp
    pub last_attempt: Option<std::time::SystemTime>,

    /// Last successful connection timestamp
    pub last_connected: Option<std::time::SystemTime>,

    /// Number of connection attempts
    pub attempts: u32,

    /// Last error message (if any)
    pub last_error: Option<String>,
}

impl PubConnection {
    /// Create a new pub connection (not yet connected)
    pub fn new(address: PubAddress) -> Self {
        Self {
            address,
            connected: false,
            last_attempt: None,
            last_connected: None,
            attempts: 0,
            last_error: None,
        }
    }

    /// Attempt to connect to the pub server
    pub async fn connect(&mut self, _keypair: &SSBKeypair) -> Result<()> {
        use std::time::SystemTime;

        self.last_attempt = Some(SystemTime::now());
        self.attempts += 1;

        tracing::debug!(
            "Attempting to connect to pub {} (attempt {})",
            self.address.socket_addr(),
            self.attempts
        );

        let socket_addr = self.address.socket_addr();
        let stream = tokio::net::TcpStream::connect(&socket_addr).await.map_err(
            |e| -> crate::error::PlurcastError {
                let error_msg = format!("Failed to connect to pub {}: {}", socket_addr, e);
                self.last_error = Some(error_msg.clone());
                self.connected = false;

                tracing::warn!("{}", error_msg);

                PlatformError::Authentication(error_msg).into()
            },
        )?;

        tracing::debug!("TCP connection established to {}", socket_addr);

        // TODO: Implement SSB handshake using kuska-ssb

        self.connected = true;
        self.last_connected = Some(SystemTime::now());
        self.last_error = None;

        drop(stream);

        tracing::info!(
            "Successfully connected to pub {} (attempt {})",
            socket_addr,
            self.attempts
        );

        Ok(())
    }

    /// Check if the connection is currently active
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// Get the time since last successful connection
    pub fn time_since_connected(&self) -> Option<std::time::Duration> {
        self.last_connected
            .and_then(|t| std::time::SystemTime::now().duration_since(t).ok())
    }

    /// Disconnect from the pub server
    pub fn disconnect(&mut self) {
        if self.connected {
            tracing::debug!("Disconnecting from pub {}", self.address.socket_addr());
            self.connected = false;
        }
    }

    /// Check if reconnection should be attempted
    pub fn should_reconnect(&self) -> bool {
        if self.connected {
            return false;
        }

        if self.last_attempt.is_none() {
            return true;
        }

        let backoff_secs = std::cmp::min(2u64.pow(self.attempts.saturating_sub(1)), 60);
        let backoff = std::time::Duration::from_secs(backoff_secs);

        if let Some(last_attempt) = self.last_attempt {
            if let Ok(elapsed) = std::time::SystemTime::now().duration_since(last_attempt) {
                return elapsed >= backoff;
            }
        }

        false
    }

    /// Reset connection state for retry
    pub fn reset_attempts(&mut self) {
        self.attempts = 0;
        self.last_error = None;
    }
}
