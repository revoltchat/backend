use fastwebsockets::{Frame, OpCode, Payload, upgrade::upgrade};
use futures::channel::oneshot::Sender;
use revolt_database::events::client::ReadyPayloadFields;
use revolt_result::{create_error, Result};
use serde::{Deserialize, Serialize};

/// Enumeration of supported protocol formats
#[derive(Debug)]
pub enum ProtocolFormat {
    Json,
    Msgpack,
}

/// User-provided protocol configuration
#[derive(Debug)]
pub struct ProtocolConfiguration {
    pub(crate) protocol_version: i32,
    pub(crate) format: ProtocolFormat,
    pub(crate) session_token: Option<String>,
}

impl ProtocolConfiguration {
    /// Create a new protocol configuration object from provided data
    pub fn from(
        protocol_version: i32,
        format: ProtocolFormat,
        session_token: Option<String>,
    ) -> Self {
        Self {
            protocol_version,
            format,
            session_token,
        }
    }

    /// Decode some WebSocket message into a T: Deserialize using the client's specified protocol format
    pub fn decode<'a, T: Deserialize<'a>>(&self, frame: &'a Frame) -> Result<T> {
        match self.format {
            ProtocolFormat::Json => {
                if let OpCode::Text = frame.opcode {
                    serde_json::from_slice(&frame.payload).map_err(|_| create_error!(InternalError))
                } else {
                    Err(create_error!(InternalError))
                }
            }
            ProtocolFormat::Msgpack => {
                if let OpCode::Binary = frame.opcode {
                    rmp_serde::from_slice(&frame.payload).map_err(|_| create_error!(InternalError))
                } else {
                    Err(create_error!(InternalError))
                }
            }
        }
    }

    /// Encode T: Serialize into a WebSocket message using the client's specified protocol format
    pub fn encode<T: Serialize>(&self, data: &T) -> Frame {
        match self.format {
            ProtocolFormat::Json => {
                let payload = serde_json::to_vec(data).expect("Failed to serialise (as json).");
                Frame::new(true, OpCode::Text, None, Payload::Owned(payload))
            }
            ProtocolFormat::Msgpack => {
                let payload = rmp_serde::to_vec_named(data).expect("Failed to serialise (as msgpack).");
                Frame::new(true, OpCode::Binary, None, Payload::Owned(payload))
            }
        }
    }

    /// Set the current session token
    pub fn set_session_token(&mut self, token: String) {
        self.session_token.replace(token);
    }

    /// Get the current session token
    pub fn get_session_token(&self) -> &Option<String> {
        &self.session_token
    }

    /// Get the protocol version specified
    pub fn get_protocol_version(&self) -> i32 {
        self.protocol_version
    }

    /// Get the protocol format specified
    pub fn get_protocol_format(&self) -> &ProtocolFormat {
        &self.format
    }

    /// Get ready payload fields
    pub fn get_ready_payload_fields(&self) -> Vec<ReadyPayloadFields> {
        vec![
            ReadyPayloadFields::Users,
            ReadyPayloadFields::Servers,
            ReadyPayloadFields::Channels,
            ReadyPayloadFields::Members,
            ReadyPayloadFields::Emoji,
        ]
    }
}
