/// crawlds-ipc: Shared types, event models, and error envelope.
pub mod error;
pub mod events;
pub mod types;

pub use error::{CrawlError, CrawlResult, ErrorEnvelope};
pub use events::{CrawlEvent, ThemeEvent};
pub use types::{
    TerminalColorSet, TerminalColors, ThemeColors, ThemeData, ThemeMetadata, ThemeSchemeType,
    ThemeVariant,
};
