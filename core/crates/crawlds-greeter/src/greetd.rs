//! greetd IPC communication
//!
//! Handles communication with the greetd greeter daemon via its IPC protocol.

use crate::types::{GreeterMessageType, GreeterState, GreeterStatus};
use greetd_ipc::{codec::TokioCodec, AuthMessageType, ErrorType, Request, Response};
use std::path::Path;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::net::UnixStream;

#[derive(Debug, Error)]
pub enum GreeterError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("greetd IPC error: {0}")]
    Ipc(String),

    #[error("session expired")]
    SessionExpired,

    #[error("no active session")]
    NoSession,
}

impl From<greetd_ipc::codec::Error> for GreeterError {
    fn from(e: greetd_ipc::codec::Error) -> Self {
        GreeterError::Ipc(e.to_string())
    }
}

#[derive(Debug)]
pub struct GreeterSession {
    pub username: String,
    pub status: GreeterStatus,
    pub last_activity: Instant,
    stream: UnixStream,
}

impl GreeterSession {
    pub async fn create(socket_path: &str, username: String) -> Result<Self, GreeterError> {
        let path = Path::new(socket_path);
        let stream = UnixStream::connect(path).await?;

        let mut session = Self {
            username: username.clone(),
            status: GreeterStatus {
                state: GreeterState::Authenticating,
                username: username.clone(),
                message: None,
                message_type: None,
                last_error: None,
            },
            last_activity: Instant::now(),
            stream,
        };

        session.create_session().await?;
        Ok(session)
    }

    async fn create_session(&mut self) -> Result<(), GreeterError> {
        let req = Request::CreateSession {
            username: self.username.clone(),
        };
        req.write_to(&mut self.stream).await?;

        let resp = Response::read_from(&mut self.stream).await
            .map_err(|e| GreeterError::Ipc(e.to_string()))?;

        match resp {
            Response::Success => {
                self.status.state = GreeterState::Ready;
            }
            Response::AuthMessage {
                auth_message_type,
                auth_message,
            } => {
                self.status.state = GreeterState::AwaitingInput;
                self.status.message = Some(auth_message);
                self.status.message_type = Some(map_msg_type(&auth_message_type));
            }
            Response::Error { error_type, description } => {
                self.status.state = GreeterState::Error;
                self.status.last_error = Some(map_error(&error_type, description));
            }
        }

        Ok(())
    }

    pub async fn respond(&mut self, response: Option<String>) -> Result<Response, GreeterError> {
        let req = Request::PostAuthMessageResponse { response };
        req.write_to(&mut self.stream).await?;

        let resp = Response::read_from(&mut self.stream).await
            .map_err(|e| GreeterError::Ipc(e.to_string()))?;

        self.last_activity = Instant::now();
        self.update_status(&resp);

        Ok(resp)
    }

    pub async fn cancel(&mut self) -> Result<(), GreeterError> {
        let req = Request::CancelSession;
        req.write_to(&mut self.stream).await?;

        let _ = Response::read_from(&mut self.stream).await;
        self.last_activity = Instant::now();

        Ok(())
    }

    pub async fn start(&mut self, cmd: Vec<String>, env: Vec<String>) -> Result<Response, GreeterError> {
        let req = Request::StartSession { cmd, env };
        req.write_to(&mut self.stream).await?;

        let resp = Response::read_from(&mut self.stream).await
            .map_err(|e| GreeterError::Ipc(e.to_string()))?;

        self.last_activity = Instant::now();
        Ok(resp)
    }

    fn update_status(&mut self, resp: &Response) {
        match resp {
            Response::Success => {
                self.status.state = GreeterState::Ready;
            }
            Response::AuthMessage {
                auth_message_type,
                auth_message,
            } => {
                self.status.state = GreeterState::AwaitingInput;
                self.status.message = Some(auth_message.clone());
                self.status.message_type = Some(map_msg_type(auth_message_type));
            }
            Response::Error { error_type, description } => {
                self.status.state = GreeterState::Error;
                self.status.last_error = Some(map_error(error_type, description.clone()));
            }
        }
    }

    pub fn status(&self) -> &GreeterStatus {
        &self.status
    }

    pub fn is_expired(&self, ttl: Duration) -> bool {
        self.last_activity.elapsed() > ttl
    }
}

#[derive(Debug, Default)]
pub struct GreeterManager {
    pub session: Option<GreeterSession>,
}

impl GreeterManager {
    pub fn new() -> Self {
        Self { session: None }
    }

    pub fn status(&self) -> GreeterStatus {
        self.session
            .as_ref()
            .map(|s| s.status.clone())
            .unwrap_or(GreeterStatus {
                state: GreeterState::Inactive,
                username: String::new(),
                message: None,
                message_type: None,
                last_error: None,
            })
    }

    pub fn is_expired(&self, ttl: Duration) -> bool {
        self.session
            .as_ref()
            .map(|s| s.is_expired(ttl))
            .unwrap_or(false)
    }

    pub fn clear_session(&mut self) {
        self.session = None;
    }
}

fn map_msg_type(kind: &AuthMessageType) -> GreeterMessageType {
    match kind {
        AuthMessageType::Visible => GreeterMessageType::Visible,
        AuthMessageType::Secret => GreeterMessageType::Secret,
        AuthMessageType::Info => GreeterMessageType::Info,
        AuthMessageType::Error => GreeterMessageType::Error,
    }
}

fn map_error(kind: &ErrorType, description: String) -> String {
    match kind {
        ErrorType::AuthError => format!("auth_error: {description}"),
        ErrorType::Error => format!("error: {description}"),
    }
}
