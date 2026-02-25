use std::{
    env,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    str::FromStr,
};

use mythic_grpc::{
    PushC2MessageFromAgent,
    client::{ClientConfig, MythicGrpcClient},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use tracing::{error, info, warn};

use crate::error::ServerError;

const C2_PROFILE_NAME: &str = "reverse_tcp";
/// Number of bytes used as the length prefix on each TCP message.
const LENGTH_PREFIX_BYTES: usize = 4;

#[derive(Debug)]
pub struct Server {
    port: u16,
    ip: IpAddr,
}

impl Server {
    pub fn new() -> Result<Self, ServerError> {
        let ip = env::var("LISTEN_ADDR").unwrap_or("0.0.0.0".to_string());

        let ip = match Ipv4Addr::from_str(ip.as_str()) {
            Ok(value) => IpAddr::from(value),
            Err(_) => match Ipv6Addr::from_str(ip.as_str()) {
                Ok(value) => IpAddr::from(value),
                Err(_) => return Err(ServerError::InvalidIpAddress(ip.into())),
            },
        };

        let port = env::var("LISTEN_PORT").unwrap_or("4444".to_string());
        let port = match port.parse::<u16>() {
            Ok(value) => value,
            Err(_) => return Err(ServerError::InvalidPort(port.into())),
        };

        Ok(Server { port, ip })
    }

    pub async fn start(&self) -> Result<(), ServerError> {
        let listener = TcpListener::bind(format!("{}:{}", self.ip, self.port))
            .await
            .map_err(|e| {
                let err = ServerError::ListenError(e);
                error!(error = %err, "socket bind error");
                err
            })?;

        info!(addr = %listener.local_addr().unwrap(), "listening for connections");

        loop {
            match listener.accept().await {
                Ok((stream, peer)) => {
                    info!(?peer, "new client connected");
                    tokio::spawn(handle_connection(stream, peer.ip().to_string()));
                }
                Err(e) => {
                    error!(error = %e, "error accepting connection");
                }
            }
        }
    }
}

impl Default for Server {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

async fn handle_connection(stream: TcpStream, remote_ip: String) {
    let config = ClientConfig::default();
    let grpc_client = MythicGrpcClient::new(config);

    let (tx, mut rx) = match grpc_client.start_push_c2_streaming().await {
        Ok(pair) => pair,
        Err(e) => {
            error!(error = %e, "failed to open PushC2 stream");
            return;
        }
    };

    info!(?remote_ip, "PushC2 stream opened");

    // Agent → Mythic: read framed messages from TCP, forward to gRPC
    let tx_clone = tx.clone();
    let remote_ip_clone = remote_ip.clone();
    let (mut tcp_read, mut tcp_write) = stream.into_split();

    // Spawn a task to read from TCP and send to Mythic
    let agent_to_mythic = tokio::spawn(async move {
        let mut len_buf = [0u8; LENGTH_PREFIX_BYTES];
        loop {
            // Read the 4-byte length prefix
            if let Err(e) = tcp_read.read_exact(&mut len_buf).await {
                if e.kind() != std::io::ErrorKind::UnexpectedEof {
                    warn!(error = %e, "error reading length prefix from agent");
                }
                break;
            }
            let msg_len = u32::from_be_bytes(len_buf) as usize;

            // Read the base64 payload
            let mut payload = vec![0u8; msg_len];
            if let Err(e) = tcp_read.read_exact(&mut payload).await {
                warn!(error = %e, "error reading payload from agent");
                break;
            }

            info!(
                len = msg_len,
                data = %String::from_utf8_lossy(&payload),
                "received data from agent"
            );

            let msg = PushC2MessageFromAgent {
                c2_profile_name: C2_PROFILE_NAME.to_string(),
                remote_ip: remote_ip_clone.clone(),
                base64_message: payload,
                ..Default::default()
            };

            if tx_clone.send(msg).await.is_err() {
                // gRPC stream closed
                break;
            }
        }
    });

    // Mythic → Agent: read from gRPC and write back to TCP
    let mythic_to_agent = tokio::spawn(async move {
        while let Some(response) = rx.recv().await {
            if !response.success {
                warn!(error = %response.error, "Mythic returned error for message");
                // keep connection alive; the agent may retry
                continue;
            }

            let payload = response.message;
            let len = payload.len() as u32;
            let len_bytes = len.to_be_bytes();

            if tcp_write.write_all(&len_bytes).await.is_err() {
                break;
            }
            if tcp_write.write_all(&payload).await.is_err() {
                break;
            }
        }
    });

    // Wait for either direction to finish, then clean up both
    tokio::select! {
        _ = agent_to_mythic => {
            info!("agent_to_mythic finished")
        }
        _ = mythic_to_agent => {
            info!("mythic_to_agent finished")
        }
    }

    info!(?remote_ip, "connection closed");
}
