//! crawlds-greeter: Greeter integration with greetd and PAM support.
//!
//! This crate provides:
//! - greetd IPC communication
//! - PAM stack detection and configuration parsing
//! - External authentication support (fprintd, U2F)
//! - Session memory persistence
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    crawlds-greeter                         │
//! │                                                          │
//! │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐  │
//! │  │   greetd    │  │    pam      │  │    external     │  │
//! │  │             │  │  detection   │  │  (fprintd/U2F)  │  │
//! │  └─────────────┘  └─────────────┘  └─────────────────┘  │
//! │          │                │                  │         │
//! │          └────────────────┼──────────────────┘         │
//! │                           │                            │
//! │  ┌─────────────────────────────────────────────────────┐│
//! │  │                    GreeterState                      ││
//! │  │   • Session management                               ││
//! │  │   • Auth state machine                               ││
//! │  │   • Memory persistence                              ││
//! │  └─────────────────────────────────────────────────────┘│
//! └─────────────────────────────────────────────────────────────┘
//!                           │
//!                    HTTP endpoints
//!                           │
//!              ┌────────────┴────────────┐
//!              │   crawlds-daemon        │
//!              │   (or standalone)       │
//!              └──────────────────────────┘
//! ```

pub mod config;
pub mod greetd;
pub mod pam;
pub mod external;
pub mod memory;
pub mod types;

pub use config::Config;
pub use greetd::{GreeterManager, GreeterSession, GreeterError};
pub use memory::{Memory, MemoryError};
pub use pam::PamStack;
pub use types::{PamInfo, ExternalAuthStatus, AuthFeedback};

// Re-export greetd_ipc types
pub use greetd_ipc::{AuthMessageType, Request, Response};
