//! Blocking WebSocket client for the live export stream.
//!
//! [`LiveClient::connect`] performs the full subscription handshake: it opens
//! the socket, sends the JSON-text `Hello`, and reads + validates the
//! server's `ServerInfo` reply. After that, [`LiveClient::next_message`]
//! yields decoded [`ServerMessage`]s (starting with that same `ServerInfo`,
//! so a state store observes every message including its `seq`).

use std::io;
use std::net::TcpStream;
use std::time::Duration;

use tungstenite::{Message, WebSocket};

use subtr_actor_live::{
    ClientMessage, Encoding, PROTOCOL_MAJOR, PROTOCOL_MINOR, ProtocolError, ServerMessage,
    protocol_versions_compatible,
};

/// Server heartbeats default to a couple of seconds, so a much larger read
/// timeout only trips when the connection is genuinely dead (or the server is
/// configured with an unusually long heartbeat interval).
const READ_TIMEOUT: Duration = Duration::from_secs(30);
const WRITE_TIMEOUT: Duration = Duration::from_secs(10);

/// Failure connecting to or reading from a live export server.
#[derive(Debug)]
pub enum ClientError {
    /// The URL was not a parseable `ws://host:port[/path]` address.
    InvalidUrl(String),
    Io(io::Error),
    WebSocket(Box<tungstenite::Error>),
    Protocol(ProtocolError),
    /// The server closed the connection during the handshake, usually with a
    /// protocol-version rejection reason.
    Rejected(String),
    /// The server's advertised protocol version is incompatible with this
    /// build (see the version rule in `subtr_actor_live::protocol`).
    IncompatibleServer {
        server_major: u16,
        server_minor: u16,
    },
    /// A message arrived in the wrong websocket frame type for the negotiated
    /// encoding (e.g. a text frame on a postcard subscription).
    UnexpectedEncoding {
        encoding: Encoding,
    },
    /// The handshake got a message other than `ServerInfo` first.
    UnexpectedMessage(String),
}

impl std::fmt::Display for ClientError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientError::InvalidUrl(url) => {
                write!(
                    formatter,
                    "invalid websocket url (want ws://host:port): {url}"
                )
            }
            ClientError::Io(error) => write!(formatter, "io: {error}"),
            ClientError::WebSocket(error) => write!(formatter, "websocket: {error}"),
            ClientError::Protocol(error) => write!(formatter, "protocol: {error}"),
            ClientError::Rejected(reason) => {
                write!(formatter, "server rejected subscription: {reason}")
            }
            ClientError::IncompatibleServer {
                server_major,
                server_minor,
            } => write!(
                formatter,
                "incompatible server protocol {server_major}.{server_minor}; \
                 client speaks {PROTOCOL_MAJOR}.{PROTOCOL_MINOR}"
            ),
            ClientError::UnexpectedEncoding { encoding } => write!(
                formatter,
                "websocket frame type does not match negotiated encoding {encoding:?}"
            ),
            ClientError::UnexpectedMessage(context) => {
                write!(formatter, "unexpected message: {context}")
            }
        }
    }
}

impl std::error::Error for ClientError {}

impl From<io::Error> for ClientError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<tungstenite::Error> for ClientError {
    fn from(error: tungstenite::Error) -> Self {
        Self::WebSocket(Box::new(error))
    }
}

impl From<ProtocolError> for ClientError {
    fn from(error: ProtocolError) -> Self {
        Self::Protocol(error)
    }
}

/// Host/port/path parts of a `ws://` URL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WsUrlParts {
    pub host: String,
    pub port: u16,
    pub path: String,
}

/// Parses `ws://host:port[/path]`. TLS (`wss://`) is not supported; the live
/// export server is a plain local socket.
pub(crate) fn parse_ws_url(url: &str) -> Result<WsUrlParts, ClientError> {
    let invalid = || ClientError::InvalidUrl(url.to_owned());
    let rest = url.strip_prefix("ws://").ok_or_else(invalid)?;
    let (authority, path) = match rest.find('/') {
        Some(index) => (&rest[..index], &rest[index..]),
        None => (rest, "/"),
    };
    let (host, port) = authority.rsplit_once(':').ok_or_else(invalid)?;
    if host.is_empty() {
        return Err(invalid());
    }
    let port: u16 = port.parse().map_err(|_| invalid())?;
    Ok(WsUrlParts {
        host: host.to_owned(),
        port,
        path: path.to_owned(),
    })
}

/// Blocking client subscribed to a live export server.
pub struct LiveClient {
    ws: WebSocket<TcpStream>,
    encoding: Encoding,
    server_name: String,
    server_protocol: (u16, u16),
    /// The validated `ServerInfo`, re-yielded by the first `next_message`
    /// call so consumers see the complete message stream.
    pending: Option<ServerMessage>,
}

impl LiveClient {
    /// Connects, sends the `Hello`, and validates the server's `ServerInfo`.
    ///
    /// `encoding` selects how the server encodes its messages for this
    /// client; `max_frame_hz` caps event-free `Frame` delivery (see the
    /// protocol docs).
    pub fn connect(
        url: &str,
        encoding: Encoding,
        max_frame_hz: Option<f32>,
    ) -> Result<Self, ClientError> {
        let parts = parse_ws_url(url)?;
        let stream = TcpStream::connect((parts.host.as_str(), parts.port))?;
        stream.set_read_timeout(Some(READ_TIMEOUT))?;
        stream.set_write_timeout(Some(WRITE_TIMEOUT))?;
        let request = format!("ws://{}:{}{}", parts.host, parts.port, parts.path);
        let (mut ws, _response) =
            tungstenite::client::client(request.as_str(), stream).map_err(|error| match error {
                tungstenite::HandshakeError::Failure(error) => ClientError::from(error),
                tungstenite::HandshakeError::Interrupted(_) => ClientError::Io(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "websocket handshake timed out",
                )),
            })?;

        let hello = ClientMessage::Hello {
            protocol_major: PROTOCOL_MAJOR,
            protocol_minor: PROTOCOL_MINOR,
            encoding,
            max_frame_hz,
        };
        let hello_bytes = hello.encode(Encoding::Json)?;
        let hello_text =
            String::from_utf8(hello_bytes).expect("json encoding always produces utf-8");
        ws.send(Message::text(hello_text))?;

        let info = read_server_message(&mut ws, encoding)?.ok_or_else(|| {
            ClientError::Rejected("connection closed before ServerInfo".to_owned())
        })?;
        let ServerMessage::ServerInfo {
            protocol_major,
            protocol_minor,
            ref server,
            ..
        } = info
        else {
            return Err(ClientError::UnexpectedMessage(format!(
                "expected ServerInfo first, got {info:?}"
            )));
        };
        if !protocol_versions_compatible(encoding, protocol_major, protocol_minor) {
            return Err(ClientError::IncompatibleServer {
                server_major: protocol_major,
                server_minor: protocol_minor,
            });
        }
        let server_name = server.clone();
        Ok(Self {
            ws,
            encoding,
            server_name,
            server_protocol: (protocol_major, protocol_minor),
            pending: Some(info),
        })
    }

    /// The encoding negotiated in the `Hello`.
    pub fn encoding(&self) -> Encoding {
        self.encoding
    }

    /// Server name from the validated `ServerInfo`.
    pub fn server_name(&self) -> &str {
        &self.server_name
    }

    /// `(major, minor)` protocol version from the validated `ServerInfo`.
    pub fn server_protocol(&self) -> (u16, u16) {
        self.server_protocol
    }

    /// Reads the next server message.
    ///
    /// Returns `Ok(None)` on a clean close. Ping/pong frames are handled
    /// internally. The first call yields the `ServerInfo` validated during
    /// [`connect`](Self::connect).
    pub fn next_message(&mut self) -> Result<Option<ServerMessage>, ClientError> {
        if let Some(pending) = self.pending.take() {
            return Ok(Some(pending));
        }
        read_server_message(&mut self.ws, self.encoding)
    }
}

/// Reads until one decodable server message, a clean close (`Ok(None)`), or an
/// error. Pings are answered (tungstenite queues the pong; flushing sends it).
fn read_server_message(
    ws: &mut WebSocket<TcpStream>,
    encoding: Encoding,
) -> Result<Option<ServerMessage>, ClientError> {
    loop {
        let message = match ws.read() {
            Ok(message) => message,
            Err(tungstenite::Error::ConnectionClosed | tungstenite::Error::AlreadyClosed) => {
                return Ok(None);
            }
            Err(error) => return Err(error.into()),
        };
        match message {
            Message::Binary(bytes) => {
                if encoding != Encoding::Postcard {
                    return Err(ClientError::UnexpectedEncoding { encoding });
                }
                return Ok(Some(ServerMessage::decode(encoding, &bytes)?));
            }
            Message::Text(text) => {
                if encoding != Encoding::Json {
                    return Err(ClientError::UnexpectedEncoding { encoding });
                }
                return Ok(Some(ServerMessage::decode(encoding, text.as_bytes())?));
            }
            Message::Ping(_) | Message::Pong(_) => {
                // The pong reply is queued internally; push it out.
                let _ = ws.flush();
            }
            Message::Close(frame) => {
                let reason = frame
                    .map(|frame| frame.reason.to_string())
                    .filter(|reason| !reason.is_empty());
                return match reason {
                    Some(reason) => Err(ClientError::Rejected(reason)),
                    None => Ok(None),
                };
            }
            Message::Frame(_) => {
                return Err(ClientError::UnexpectedMessage(
                    "raw frame from websocket read".to_owned(),
                ));
            }
        }
    }
}

#[cfg(test)]
#[path = "client_tests.rs"]
mod tests;
