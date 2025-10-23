//! Shared numbering utilities for lists and sessions
//!
//! Both lists and sessions use sequence markers (1., a., i., etc.) and need to
//! convert from syntax-level types to AST-level types. This module provides
//! shared conversion logic.

use crate::ast::elements::list::{NumberingForm, NumberingStyle};
use crate::syntax::list_detection;

/// Convert syntax-level numbering style to AST-level numbering style
///
/// This is used by both lists and sessions when extracting numbering from
/// sequence markers. The conversion is straightforward except that:
/// - Lists support Plain style (dash markers)
/// - Sessions do NOT support Plain style (returns None)
pub fn convert_numbering_style(
    style: &list_detection::NumberingStyle,
    allow_plain: bool,
) -> Option<NumberingStyle> {
    match style {
        list_detection::NumberingStyle::Plain => {
            if allow_plain {
                Some(NumberingStyle::Plain)
            } else {
                None // Sessions don't allow plain markers
            }
        }
        list_detection::NumberingStyle::Numerical => Some(NumberingStyle::Numerical),
        list_detection::NumberingStyle::Alphabetical => Some(NumberingStyle::Alphabetical),
        list_detection::NumberingStyle::Roman => Some(NumberingStyle::Roman),
    }
}

/// Convert syntax-level numbering form to AST-level numbering form
///
/// Regular form (single-level like "1.") becomes Short
/// Extended form (hierarchical like "1.2.3.") becomes Full
pub fn convert_numbering_form(form: &list_detection::NumberingForm) -> NumberingForm {
    match form {
        list_detection::NumberingForm::Regular => NumberingForm::Short,
        list_detection::NumberingForm::Extended => NumberingForm::Full,
    }
}
