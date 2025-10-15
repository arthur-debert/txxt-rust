//! Legacy structure module - re-exports from elements
//!
//! This module provides backward compatibility for the old structure module.
//! The actual implementation is now distributed across elements/ submodules.

pub use crate::ast::elements::{
    containers::ContentContainer,
    core::{BlankLine, ContainerType},
    list::{NumberingForm, NumberingStyle},
    paragraph::ParagraphBlock,
    session::SessionContainer,
    session::{SessionBlock, SessionNumbering, SessionTitle},
    verbatim::IgnoreContainer,
};
