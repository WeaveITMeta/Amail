//! SMTP Server and Client implementation
//!
//! Provides inbound SMTP server for receiving emails and outbound client for sending.

use crate::config::MailConfig;
use crate::events::MailEvent;
use anyhow::Result;
use mail_parser::MessageParser;
use tokio::sync::mpsc;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{info, warn, error, debug};
use std::sync::Arc;

// =============================================================================
// SMTP Server (Inbound)
// =============================================================================

/// Embedded SMTP server for receiving emails
pub struct SmtpServer {
    config: MailConfig,
    event_tx: mpsc::Sender<MailEvent>,
}

impl SmtpServer {
    /// Create a new SMTP server
    pub fn new(config: MailConfig, event_tx: mpsc::Sender<MailEvent>) -> Self {
        Self { config, event_tx }
    }

    /// Start the SMTP server
    pub async fn start(self) -> Result<()> {
        let bind_addr = self.config.smtp_bind.to_string();
        info!("Starting SMTP server on {}", bind_addr);

        let listener = TcpListener::bind(&bind_addr).await?;
        let config = Arc::new(self.config);
        let event_tx = self.event_tx;

        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        debug!("SMTP connection from {}", addr);
                        let config = config.clone();
                        let tx = event_tx.clone();
                        tokio::spawn(async move {
                            if let Err(e) = handle_smtp_connection(stream, config, tx).await {
                                warn!("SMTP session error: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        error!("SMTP accept error: {}", e);
                    }
                }
            }
        });

        Ok(())
    }
}

/// Handle a single SMTP connection
async fn handle_smtp_connection(
    stream: TcpStream,
    config: Arc<MailConfig>,
    event_tx: mpsc::Sender<MailEvent>,
) -> Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    // Session state
    let mut mail_from: Option<String> = None;
    let mut rcpt_to: Vec<String> = Vec::new();
    let mut in_data = false;
    let mut data_buffer: Vec<u8> = Vec::new();

    // Send greeting
    writer.write_all(format!("220 {} ESMTP Amail\r\n", config.hostname).as_bytes()).await?;

    loop {
        line.clear();
        let bytes_read = reader.read_line(&mut line).await?;
        
        if bytes_read == 0 {
            break; // Connection closed
        }

        let line = line.trim();
        debug!("SMTP recv: {}", line);

        if in_data {
            // Collecting message data
            if line == "." {
                // End of data
                in_data = false;
                
                // Process the message
                if let Some(from) = &mail_from {
                    process_message(&data_buffer, from, &rcpt_to, &config, &event_tx).await;
                }
                
                writer.write_all(b"250 OK message queued\r\n").await?;
                
                // Reset for next message
                mail_from = None;
                rcpt_to.clear();
                data_buffer.clear();
            } else {
                // Handle dot-stuffing
                let content = if line.starts_with("..") {
                    &line[1..]
                } else {
                    line
                };
                data_buffer.extend_from_slice(content.as_bytes());
                data_buffer.extend_from_slice(b"\r\n");
            }
            continue;
        }

        // Parse SMTP commands
        let cmd = line.to_uppercase();
        
        if cmd.starts_with("HELO") || cmd.starts_with("EHLO") {
            let response = if cmd.starts_with("EHLO") {
                format!(
                    "250-{}\r\n250-SIZE {}\r\n250-8BITMIME\r\n250 OK\r\n",
                    config.hostname, config.max_message_size
                )
            } else {
                format!("250 {} Hello\r\n", config.hostname)
            };
            writer.write_all(response.as_bytes()).await?;
        } else if cmd.starts_with("MAIL FROM:") {
            let from = extract_address(&line[10..]);
            mail_from = Some(from);
            writer.write_all(b"250 OK\r\n").await?;
        } else if cmd.starts_with("RCPT TO:") {
            let to = extract_address(&line[8..]);
            if rcpt_to.len() < config.max_recipients {
                rcpt_to.push(to);
                writer.write_all(b"250 OK\r\n").await?;
            } else {
                writer.write_all(b"452 Too many recipients\r\n").await?;
            }
        } else if cmd == "DATA" {
            if mail_from.is_some() && !rcpt_to.is_empty() {
                in_data = true;
                data_buffer.clear();
                writer.write_all(b"354 Start mail input; end with <CRLF>.<CRLF>\r\n").await?;
            } else {
                writer.write_all(b"503 Bad sequence of commands\r\n").await?;
            }
        } else if cmd == "RSET" {
            mail_from = None;
            rcpt_to.clear();
            data_buffer.clear();
            writer.write_all(b"250 OK\r\n").await?;
        } else if cmd == "NOOP" {
            writer.write_all(b"250 OK\r\n").await?;
        } else if cmd == "QUIT" {
            writer.write_all(b"221 Bye\r\n").await?;
            break;
        } else if cmd.starts_with("AUTH") {
            // Basic AUTH support (accept anything for now)
            writer.write_all(b"235 Authentication successful\r\n").await?;
        } else {
            writer.write_all(b"500 Command not recognized\r\n").await?;
        }
    }

    Ok(())
}

/// Extract email address from MAIL FROM: or RCPT TO: parameter
fn extract_address(s: &str) -> String {
    let s = s.trim();
    if s.starts_with('<') && s.contains('>') {
        s[1..s.find('>').unwrap()].to_string()
    } else {
        s.split_whitespace().next().unwrap_or("").to_string()
    }
}

/// Process received message
async fn process_message(
    data: &[u8],
    from: &str,
    to: &[String],
    _config: &MailConfig,
    event_tx: &mpsc::Sender<MailEvent>,
) {
    let parser = MessageParser::default();
    
    let (subject, body_text, body_html, message_id) = match parser.parse(data) {
        Some(msg) => (
            msg.subject().unwrap_or("(no subject)").to_string(),
            msg.body_text(0).map(|s| s.to_string()),
            msg.body_html(0).map(|s| s.to_string()),
            msg.message_id().map(|s| s.to_string()),
        ),
        None => {
            warn!("Failed to parse incoming message");
            ("(parse error)".to_string(), None, None, None)
        }
    };

    info!("Received email: {} -> {:?}, Subject: {}", from, to, subject);

    let event = MailEvent::MessageReceived {
        id: uuid::Uuid::new_v4().to_string(),
        message_id,
        from: from.to_string(),
        to: to.to_vec(),
        subject,
        body_text,
        body_html,
        raw_size: data.len(),
        received_at: chrono::Utc::now(),
    };

    if let Err(e) = event_tx.try_send(event) {
        error!("Failed to send mail event: {}", e);
    }
}

// =============================================================================
// SMTP Client (Outbound)
// =============================================================================

/// SMTP client for sending emails
pub struct SmtpClient {
    config: MailConfig,
}

impl SmtpClient {
    /// Create a new SMTP client
    pub fn new(config: MailConfig) -> Self {
        Self { config }
    }

    /// Send an email using lettre
    pub async fn send(
        &self,
        from: &str,
        to: &[&str],
        subject: &str,
        body_text: Option<&str>,
        body_html: Option<&str>,
    ) -> Result<String> {
        use lettre::{
            Message, SmtpTransport, Transport,
            message::{header::ContentType, Mailbox, MultiPart, SinglePart},
            transport::smtp::authentication::Credentials,
        };

        // Build recipient list
        let to_mailbox: Mailbox = to.first()
            .ok_or_else(|| anyhow::anyhow!("No recipients"))?
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid recipient: {}", e))?;

        // Build message
        let mut message_builder = Message::builder()
            .from(from.parse().map_err(|e| anyhow::anyhow!("Invalid from: {}", e))?)
            .to(to_mailbox)
            .subject(subject);

        // Add additional recipients
        for recipient in to.iter().skip(1) {
            message_builder = message_builder.to(
                recipient.parse().map_err(|e| anyhow::anyhow!("Invalid recipient: {}", e))?
            );
        }

        // Build body
        let message = if let (Some(text), Some(html)) = (body_text, body_html) {
            message_builder.multipart(
                MultiPart::alternative()
                    .singlepart(SinglePart::builder()
                        .header(ContentType::TEXT_PLAIN)
                        .body(text.to_string()))
                    .singlepart(SinglePart::builder()
                        .header(ContentType::TEXT_HTML)
                        .body(html.to_string()))
            )?
        } else if let Some(html) = body_html {
            message_builder.header(ContentType::TEXT_HTML).body(html.to_string())?
        } else {
            message_builder.header(ContentType::TEXT_PLAIN)
                .body(body_text.unwrap_or("").to_string())?
        };

        // Build transport
        let smtp_host = std::env::var("SMTP_HOST").unwrap_or_else(|_| self.config.hostname.clone());
        let smtp_port: u16 = std::env::var("SMTP_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(587);

        let mut transport_builder = SmtpTransport::builder_dangerous(&smtp_host)
            .port(smtp_port);

        // Add credentials if configured
        if let (Ok(username), Ok(password)) = (
            std::env::var("SMTP_USERNAME"),
            std::env::var("SMTP_PASSWORD"),
        ) {
            if !username.is_empty() && !password.is_empty() {
                transport_builder = transport_builder.credentials(
                    Credentials::new(username, password)
                );
            }
        }

        let transport = transport_builder.build();

        // Send
        transport.send(&message)
            .map_err(|e| anyhow::anyhow!("SMTP send failed: {}", e))?;

        let message_id = uuid::Uuid::new_v4().to_string();
        info!("Email sent: {} -> {:?}, ID: {}", from, to, message_id);

        Ok(message_id)
    }
}
