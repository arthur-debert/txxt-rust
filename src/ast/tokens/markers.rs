//! Marker tokens for TXXT parsing pipeline
//!
//! This module defines marker scanner tokens that represent
//! TXXT-specific structural markers and sequence markers.

use serde::{Deserialize, Serialize};

use crate::ast::elements::scanner_tokens::{Position, SequenceMarkerType, SourceSpan};

/// TXXT marker (::) - fundamental structural element
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TxxtMarkerToken {
    /// Source span of the marker
    pub span: SourceSpan,
}

/// List/sequence markers (1., -, a), etc.) with rich semantic information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SequenceMarkerToken {
    /// The sequence marker type information
    pub marker_type: SequenceMarkerType,
    /// Source span of the marker
    pub span: SourceSpan,
}
