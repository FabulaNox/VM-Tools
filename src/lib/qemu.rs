use std::collections::HashMap;
use tokio::net::UnixStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde_json::{json, Value};

use crate::lib::error::{VmError, Result};

pub struct QemuMonitor {
    socket_path: String,
}

impl QemuMonitor {
    pub fn new(socket_path: &str) -> Self {
        Self {
            socket_path: socket_path.to_string(),
        }
    }

    pub async fn connect(&self) -> Result<QemuConnection> {
        let stream = UnixStream::connect(&self.socket_path)
            .await
            .map_err(|e| VmError::QemuError(format!("Failed to connect to QEMU monitor: {}", e)))?;

        Ok(QemuConnection { stream })
    }
}

pub struct QemuConnection {
    stream: UnixStream,
}

impl QemuConnection {
    pub async fn execute_command(&mut self, command: &str) -> Result<Value> {
        // Send QMP command
        let qmp_command = json!({
            "execute": command,
            "arguments": {}
        });

        let command_str = format!("{}\n", qmp_command.to_string());
        self.stream.write_all(command_str.as_bytes())
            .await
            .map_err(|e| VmError::QemuError(format!("Failed to send command: {}", e)))?;

        // Read response
        let mut buffer = vec![0; 4096];
        let n = self.stream.read(&mut buffer)
            .await
            .map_err(|e| VmError::QemuError(format!("Failed to read response: {}", e)))?;

        let response = String::from_utf8_lossy(&buffer[..n]);
        let json_response: Value = serde_json::from_str(&response)
            .map_err(|e| VmError::QemuError(format!("Failed to parse response: {}", e)))?;

        Ok(json_response)
    }

    pub async fn get_vm_status(&mut self) -> Result<HashMap<String, Value>> {
        let response = self.execute_command("query-status").await?;
        
        if let Some(result) = response.get("return") {
            let mut status = HashMap::new();
            if let Some(obj) = result.as_object() {
                for (key, value) in obj {
                    status.insert(key.clone(), value.clone());
                }
            }
            Ok(status)
        } else {
            Err(VmError::QemuError("Invalid response format".to_string()))
        }
    }

    pub async fn get_cpu_stats(&mut self) -> Result<f64> {
        // This would implement CPU usage monitoring via QMP
        // For now, return a placeholder
        Ok(0.0)
    }

    pub async fn get_memory_stats(&mut self) -> Result<(u64, u64)> {
        // This would implement memory usage monitoring via QMP
        // Returns (used, total) in bytes
        Ok((0, 0))
    }

    pub async fn screenshot(&mut self, filename: &str) -> Result<()> {
        let command = json!({
            "execute": "screendump",
            "arguments": {
                "filename": filename
            }
        });

        let command_str = format!("{}\n", command.to_string());
        self.stream.write_all(command_str.as_bytes())
            .await
            .map_err(|e| VmError::QemuError(format!("Failed to take screenshot: {}", e)))?;

        Ok(())
    }

    pub async fn send_key(&mut self, key: &str) -> Result<()> {
        let command = json!({
            "execute": "send-key",
            "arguments": {
                "keys": [key]
            }
        });

        let command_str = format!("{}\n", command.to_string());
        self.stream.write_all(command_str.as_bytes())
            .await
            .map_err(|e| VmError::QemuError(format!("Failed to send key: {}", e)))?;

        Ok(())
    }
}