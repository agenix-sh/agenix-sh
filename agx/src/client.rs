use anyhow::{Context, Result};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub struct AgqClient {
    stream: TcpStream,
}

impl AgqClient {
    pub async fn connect(addr: &str) -> Result<Self> {
        let stream = TcpStream::connect(addr)
            .await
            .context(format!("Failed to connect to AGQ at {}", addr))?;
        Ok(Self { stream })
    }

    pub async fn submit_plan(&mut self, plan_json: &str) -> Result<String> {
        // Format: *2\r\n$11\r\nPLAN.SUBMIT\r\n$<len>\r\n<json>\r\n
        let cmd = format!(
            "*2\r\n$11\r\nPLAN.SUBMIT\r\n${}\r\n{}\r\n",
            plan_json.len(),
            plan_json
        );

        self.stream.write_all(cmd.as_bytes()).await?;

        // Read response
        // Expecting: $36\r\n<uuid>\r\n (BulkString) or +OK\r\n (SimpleString) or -Error\r\n
        let mut buf = [0u8; 1024];
        let n = self.stream.read(&mut buf).await?;
        let response = String::from_utf8_lossy(&buf[..n]);

        if response.starts_with('-') {
            return Err(anyhow::anyhow!("AGQ Error: {}", response.trim()));
        }

        if response.starts_with('$') {
            // Bulk string: $<len>\r\n<content>\r\n
            let parts: Vec<&str> = response.splitn(2, "\r\n").collect();
            if parts.len() < 2 {
                return Err(anyhow::anyhow!("Invalid RESP response: {}", response));
            }
            // The content is in the second part, but might be followed by \r\n
            let content = parts[1].trim();
            Ok(content.to_string())
        } else if response.starts_with('+') {
            // Simple string: +<content>\r\n
            Ok(response[1..].trim().to_string())
        } else {
            Err(anyhow::anyhow!("Unexpected RESP response: {}", response))
        }
    }
}
