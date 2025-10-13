use std::collections::HashMap;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Processing stages for test corpora.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(dead_code)] // Some variants are for future integration
pub enum ProcessingStage {
    /// Raw text as extracted from specification (default)
    #[default]
    Raw,
    /// Tokenized stream from the tokenizer
    Tokens,
    /// Block-grouped tokens after lexical analysis
    BlockedTokens,
    /// Parsed AST structure
    ParsedAst,
    /// Full document with all processing complete
    FullDocument,
}

/// A test corpus extracted from specification documents.
#[derive(Debug, Clone, PartialEq)]
pub struct Corpus {
    pub name: String,
    pub source_text: String,
    pub parameters: HashMap<String, String>,
    pub processing_stage: ProcessingStage,
    pub processed_data: Option<ProcessedData>,
}

impl Corpus {
    /// Get the processed data as tokens, if available.
    pub fn tokens(&self) -> Option<&Vec<String>> {
        match &self.processed_data {
            Some(ProcessedData::Tokens(tokens)) => Some(tokens),
            Some(ProcessedData::BlockedTokens(tokens)) => Some(tokens),
            _ => None,
        }
    }

    /// Get the processed data as AST, if available.
    pub fn ast(&self) -> Option<&str> {
        match &self.processed_data {
            Some(ProcessedData::ParsedAst(ast)) => Some(ast),
            _ => None,
        }
    }

    /// Get the processed data as a full document, if available.
    #[allow(dead_code)] // For future integration
    pub fn document(&self) -> Option<&str> {
        match &self.processed_data {
            Some(ProcessedData::FullDocument(doc)) => Some(doc),
            _ => None,
        }
    }

    /// Check if this corpus represents an error case.
    pub fn is_error_case(&self) -> bool {
        self.parameters.contains_key("error")
    }

    /// Get the expected error type, if this is an error case.
    pub fn expected_error(&self) -> Option<&str> {
        self.parameters.get("error").map(|s| s.as_str())
    }

    /// Get the expected error message, if this is an error case.
    pub fn expected_error_message(&self) -> Option<&str> {
        self.parameters.get("message").map(|s| s.as_str())
    }
}

/// Processed data from different pipeline stages.
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessedData {
    /// Raw text (no processing)
    Raw,
    /// Tokenized representation
    Tokens(Vec<String>), // Placeholder - will be replaced with actual token types
    /// Block-grouped tokens
    BlockedTokens(Vec<String>), // Placeholder - will be replaced with actual block types
    /// Parsed AST
    ParsedAst(String), // Placeholder - will be replaced with actual AST types
    /// Full document
    FullDocument(String), // Placeholder - will be replaced with actual document types
}

/// Utility for extracting test cases from TXXT specification documents.
///
/// # Overview
///
/// `TxxtCorpora` provides a spec-driven testing framework that extracts test cases
/// directly from the authoritative specification documents in `docs/specs/`. This
/// approach ensures perfect alignment between documentation and tests, preventing
/// implementation drift.
///
/// # Quick Start
///
/// ```rust
/// use tests::corpora::{TxxtCorpora, ProcessingStage};
///
/// // Load a test case (Raw text by default)
/// let corpus = TxxtCorpora::load("txxt.core.spec.paragraph.valid.simple")?;
///
/// // Load with specific processing stage
/// let corpus = TxxtCorpora::load_with_processing(
///     "txxt.core.spec.paragraph.valid.simple",
///     ProcessingStage::Tokens
/// )?;
///
/// // Access the data
/// println!("Source: {}", corpus.source_text);
/// if let Some(tokens) = corpus.tokens() {
///     println!("Tokens: {:?}", tokens);
/// }
/// ```
///
/// # Test Case Syntax
///
/// Test cases are embedded in specification documents using labeled verbatim blocks:
///
/// ```txxt
/// Simple paragraph example:
///     This is a basic paragraph containing plain text.
/// :: txxt.core.spec.paragraph.valid.simple ::
///
/// Error case with parameters:
///     - Invalid single item list
/// :: txxt.core.spec.list.error.singleItem:error="ParseError",message="Lists require multiple items" ::
/// ```
///
/// # Processing Stages
///
/// The `ProcessingStage` enum allows testing at different pipeline stages:
/// - `Raw` - Extracted text as-is (default)
/// - `Tokens` - Tokenized stream
/// - `BlockedTokens` - Block-grouped tokens
/// - `ParsedAst` - Parsed AST structure
/// - `FullDocument` - Complete document processing
///
/// # Integration for Parser Developers
///
/// **Tokenizer testing:**
/// ```rust
/// let corpus = TxxtCorpora::load_with_processing(
///     "txxt.core.spec.paragraph.valid.simple",
///     ProcessingStage::Tokens
/// )?;
/// let expected_tokens = corpus.tokens().unwrap();
/// // Compare with your tokenizer output
/// ```
///
/// **Parser testing:**
/// ```rust
/// let corpus = TxxtCorpora::load("txxt.core.spec.paragraph.valid.simple")?;
/// let ast = your_parser::parse(&corpus.source_text)?;
/// insta::assert_yaml_snapshot!(ast);
/// ```
///
/// **Error testing:**
/// ```rust
/// let corpus = TxxtCorpora::load("txxt.core.spec.list.error.singleItem")?;
/// assert!(corpus.is_error_case());
/// let result = your_parser::parse(&corpus.source_text);
/// assert!(result.is_err());
/// ```
///
/// See `tests/README.md` for complete documentation and best practices.
pub struct TxxtCorpora;

impl TxxtCorpora {
    /// Load a specific test corpus by name with default (Raw) processing.
    ///
    /// The name should follow the pattern `txxt.core.spec.*` as defined in the
    /// specification documents under `docs/specs/`.
    pub fn load(name: &str) -> Result<Corpus, CorpusError> {
        Self::load_with_processing(name, ProcessingStage::Raw)
    }

    /// Load a specific test corpus by name with specified processing stage.
    ///
    /// The name should follow the pattern `txxt.core.spec.*` as defined in the
    /// specification documents under `docs/specs/`.
    ///
    /// # Arguments
    /// * `name` - The corpus identifier (e.g., "txxt.core.spec.paragraph.valid.simple")
    /// * `processing` - The processing stage to apply to the corpus
    pub fn load_with_processing(
        name: &str,
        processing: ProcessingStage,
    ) -> Result<Corpus, CorpusError> {
        let specs_dir = Path::new("docs/specs");
        if !specs_dir.exists() {
            return Err(CorpusError::SpecsDirectoryNotFound);
        }

        for entry in WalkDir::new(specs_dir) {
            let entry = entry.map_err(|e| CorpusError::FileSystemError(e.to_string()))?;
            if entry.file_type().is_file() && entry.path().extension() == Some("txxt".as_ref()) {
                if let Ok(mut corpus) = Self::extract_from_file(entry.path(), name) {
                    Self::apply_processing(&mut corpus, processing)?;
                    return Ok(corpus);
                }
            }
        }

        Err(CorpusError::CorpusNotFound(name.to_string()))
    }

    /// Extract all corpora from the specification documents with default (Raw) processing.
    pub fn load_all() -> Result<Vec<Corpus>, CorpusError> {
        Self::load_all_with_processing(ProcessingStage::Raw)
    }

    /// Extract all corpora from the specification documents with specified processing stage.
    pub fn load_all_with_processing(
        processing: ProcessingStage,
    ) -> Result<Vec<Corpus>, CorpusError> {
        let specs_dir = Path::new("docs/specs");
        if !specs_dir.exists() {
            return Err(CorpusError::SpecsDirectoryNotFound);
        }

        let mut corpora = Vec::new();
        for entry in WalkDir::new(specs_dir) {
            let entry = entry.map_err(|e| CorpusError::FileSystemError(e.to_string()))?;
            if entry.file_type().is_file() && entry.path().extension() == Some("txxt".as_ref()) {
                let mut file_corpora = Self::extract_all_from_file(entry.path())?;
                for corpus in &mut file_corpora {
                    Self::apply_processing(corpus, processing)?;
                }
                corpora.extend(file_corpora);
            }
        }

        Ok(corpora)
    }

    /// Apply processing stage to a corpus.
    fn apply_processing(
        corpus: &mut Corpus,
        processing: ProcessingStage,
    ) -> Result<(), CorpusError> {
        corpus.processing_stage = processing;

        match processing {
            ProcessingStage::Raw => {
                corpus.processed_data = Some(ProcessedData::Raw);
            }
            ProcessingStage::Tokens => {
                // TODO: Integrate with actual tokenizer when available
                // For now, return a placeholder
                let tokens = Self::placeholder_tokenize(&corpus.source_text)?;
                corpus.processed_data = Some(ProcessedData::Tokens(tokens));
            }
            ProcessingStage::BlockedTokens => {
                // TODO: Integrate with actual block grouper when available
                let tokens = Self::placeholder_tokenize(&corpus.source_text)?;
                corpus.processed_data = Some(ProcessedData::BlockedTokens(tokens));
            }
            ProcessingStage::ParsedAst => {
                // TODO: Integrate with actual parser when available
                let placeholder_ast = format!("ParsedAST({})", corpus.source_text.len());
                corpus.processed_data = Some(ProcessedData::ParsedAst(placeholder_ast));
            }
            ProcessingStage::FullDocument => {
                // TODO: Integrate with actual document processor when available
                let placeholder_doc = format!("FullDocument({})", corpus.name);
                corpus.processed_data = Some(ProcessedData::FullDocument(placeholder_doc));
            }
        }

        Ok(())
    }

    /// Placeholder tokenizer - will be replaced with actual tokenizer integration.
    fn placeholder_tokenize(text: &str) -> Result<Vec<String>, CorpusError> {
        // Simple word-based tokenization as placeholder
        Ok(text.split_whitespace().map(|s| s.to_string()).collect())
    }

    /// Extract a specific corpus from a file.
    fn extract_from_file(file_path: &Path, target_name: &str) -> Result<Corpus, CorpusError> {
        let content = fs::read_to_string(file_path).map_err(|e| {
            CorpusError::FileReadError(file_path.to_string_lossy().to_string(), e.to_string())
        })?;

        let lines: Vec<&str> = content.lines().collect();
        let mut extractor = CorpusExtractor::new(&lines);

        while let Some(corpus) = extractor.next_corpus()? {
            if corpus.name == target_name {
                return Ok(corpus);
            }
        }

        Err(CorpusError::CorpusNotFound(target_name.to_string()))
    }

    /// Extract all corpora from a file.
    fn extract_all_from_file(file_path: &Path) -> Result<Vec<Corpus>, CorpusError> {
        let content = fs::read_to_string(file_path).map_err(|e| {
            CorpusError::FileReadError(file_path.to_string_lossy().to_string(), e.to_string())
        })?;

        let lines: Vec<&str> = content.lines().collect();
        let mut extractor = CorpusExtractor::new(&lines);
        let mut corpora = Vec::new();

        while let Some(corpus) = extractor.next_corpus()? {
            corpora.push(corpus);
        }

        Ok(corpora)
    }
}

/// State machine for extracting test cases from specification files.
struct CorpusExtractor<'a> {
    lines: &'a [&'a str],
    current_line: usize,
}

impl<'a> CorpusExtractor<'a> {
    fn new(lines: &'a [&'a str]) -> Self {
        Self {
            lines,
            current_line: 0,
        }
    }

    /// Extract the next corpus from the file, if any.
    fn next_corpus(&mut self) -> Result<Option<Corpus>, CorpusError> {
        // Scan for the next verbatim label matching txxt.core.spec.*
        while self.current_line < self.lines.len() {
            let line = self.lines[self.current_line];
            if let Some((name, parameters)) = Self::parse_corpus_label(line) {
                let corpus = self.extract_corpus_at_line(self.current_line, name, parameters)?;
                self.current_line += 1;
                return Ok(Some(corpus));
            }
            self.current_line += 1;
        }
        Ok(None)
    }

    /// Parse a verbatim label line to extract corpus name and parameters.
    fn parse_corpus_label(line: &str) -> Option<(String, HashMap<String, String>)> {
        let trimmed = line.trim();

        // Check if this is a verbatim label line with txxt.core.spec pattern
        if !trimmed.starts_with("::") || !trimmed.ends_with("::") {
            return None;
        }

        // Handle edge case of just "::"
        if trimmed.len() < 4 {
            return None;
        }

        let content = trimmed[2..trimmed.len() - 2].trim();

        if !content.starts_with("txxt.core.spec.") {
            return None;
        }

        // Parse name and parameters
        if let Some(colon_pos) = content.find(':') {
            let name = content[..colon_pos].trim().to_string();
            let params_str = &content[colon_pos + 1..];
            let parameters = Self::parse_parameters(params_str);
            Some((name, parameters))
        } else {
            let name = content.trim().to_string();
            Some((name, HashMap::new()))
        }
    }

    /// Parse parameter string like `error="ParseError",message="Lists require multiple items"`
    fn parse_parameters(params_str: &str) -> HashMap<String, String> {
        let mut parameters = HashMap::new();

        for param in params_str.split(',') {
            let param = param.trim();
            if let Some(eq_pos) = param.find('=') {
                let key = param[..eq_pos].trim().to_string();
                let value = param[eq_pos + 1..].trim();

                // Remove quotes if present
                let value = if (value.starts_with('"') && value.ends_with('"'))
                    || (value.starts_with('\'') && value.ends_with('\''))
                {
                    &value[1..value.len() - 1]
                } else {
                    value
                };

                parameters.insert(key, value.to_string());
            }
        }

        parameters
    }

    /// Extract the corpus content at the given label line.
    fn extract_corpus_at_line(
        &self,
        label_line: usize,
        name: String,
        parameters: HashMap<String, String>,
    ) -> Result<Corpus, CorpusError> {
        let label_indentation = Self::get_indentation(self.lines[label_line]);

        // Search backwards from the label to find the title line
        let mut title_line = None;
        for i in (0..label_line).rev() {
            let line = self.lines[i];
            let line_indentation = Self::get_indentation(line);

            // Skip empty lines
            if line.trim().is_empty() {
                continue;
            }

            // If we find a line at the same indentation that ends with ':', it's our title
            if line_indentation == label_indentation && line.trim().ends_with(':') {
                title_line = Some(i);
                break;
            }

            // If we find a line with less indentation, we've gone too far
            if line_indentation < label_indentation {
                break;
            }
        }

        let title_line = title_line.ok_or_else(|| CorpusError::TitleLineNotFound(name.clone()))?;

        // Extract content between title line and label line
        let mut source_lines = Vec::new();
        for i in (title_line + 1)..label_line {
            let line = self.lines[i];
            // Remove the base indentation level
            let content = Self::remove_base_indentation(line, label_indentation + 4);
            source_lines.push(content);
        }

        // Remove trailing empty lines
        while source_lines
            .last()
            .is_some_and(|line| line.trim().is_empty())
        {
            source_lines.pop();
        }

        let source_text = source_lines.join("\n");

        Ok(Corpus {
            name,
            source_text,
            parameters,
            processing_stage: ProcessingStage::Raw,
            processed_data: None,
        })
    }

    /// Get the indentation level of a line (number of leading spaces).
    fn get_indentation(line: &str) -> usize {
        line.chars().take_while(|&c| c == ' ').count()
    }

    /// Remove base indentation from a line.
    fn remove_base_indentation(line: &str, base_indent: usize) -> String {
        if line.trim().is_empty() {
            return String::new();
        }

        let line_indent = Self::get_indentation(line);
        if line_indent >= base_indent {
            line[base_indent..].to_string()
        } else {
            line.to_string()
        }
    }
}

#[derive(Debug, Clone)]
pub enum CorpusError {
    SpecsDirectoryNotFound,
    FileSystemError(String),
    FileReadError(String, String),
    CorpusNotFound(String),
    TitleLineNotFound(String),
}

impl std::fmt::Display for CorpusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CorpusError::SpecsDirectoryNotFound => {
                write!(f, "Specifications directory 'docs/specs' not found")
            }
            CorpusError::FileSystemError(err) => {
                write!(f, "File system error: {}", err)
            }
            CorpusError::FileReadError(path, err) => {
                write!(f, "Failed to read file '{}': {}", path, err)
            }
            CorpusError::CorpusNotFound(name) => {
                write!(f, "Corpus '{}' not found in specification documents", name)
            }
            CorpusError::TitleLineNotFound(name) => {
                write!(f, "Title line not found for corpus '{}'", name)
            }
        }
    }
}

impl std::error::Error for CorpusError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_corpus_label() {
        // Test basic label
        let (name, params) =
            CorpusExtractor::parse_corpus_label(":: txxt.core.spec.paragraph.valid.simple ::")
                .unwrap();
        assert_eq!(name, "txxt.core.spec.paragraph.valid.simple");
        assert!(params.is_empty());

        // Test label with parameters
        let (name, params) = CorpusExtractor::parse_corpus_label(r#":: txxt.core.spec.list.error.singleItem:error="ParseError",message="Lists require multiple items" ::"#).unwrap();
        assert_eq!(name, "txxt.core.spec.list.error.singleItem");
        assert_eq!(params.get("error"), Some(&"ParseError".to_string()));
        assert_eq!(
            params.get("message"),
            Some(&"Lists require multiple items".to_string())
        );

        // Test non-corpus label
        assert!(CorpusExtractor::parse_corpus_label(":: basic ::").is_none());
        assert!(CorpusExtractor::parse_corpus_label("regular text").is_none());
    }

    #[test]
    fn test_parse_parameters() {
        let params = CorpusExtractor::parse_parameters(
            r#"error="ParseError",message="Lists require multiple items",line=1,column=1"#,
        );
        assert_eq!(params.get("error"), Some(&"ParseError".to_string()));
        assert_eq!(
            params.get("message"),
            Some(&"Lists require multiple items".to_string())
        );
        assert_eq!(params.get("line"), Some(&"1".to_string()));
        assert_eq!(params.get("column"), Some(&"1".to_string()));
    }

    #[test]
    fn test_get_indentation() {
        assert_eq!(CorpusExtractor::get_indentation("    text"), 4);
        assert_eq!(CorpusExtractor::get_indentation("text"), 0);
        assert_eq!(CorpusExtractor::get_indentation("        more text"), 8);
    }

    #[test]
    fn test_processing_stages() {
        use super::*;

        // Test default processing stage
        assert_eq!(ProcessingStage::default(), ProcessingStage::Raw);

        // Test placeholder tokenization
        let tokens = TxxtCorpora::placeholder_tokenize("This is a test").unwrap();
        assert_eq!(tokens, vec!["This", "is", "a", "test"]);
    }

    #[test]
    fn test_corpus_convenience_methods() {
        use super::*;

        let mut corpus = Corpus {
            name: "test".to_string(),
            source_text: "test content".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert("error".to_string(), "ParseError".to_string());
                params.insert("message".to_string(), "Test error".to_string());
                params
            },
            processing_stage: ProcessingStage::Raw,
            processed_data: None,
        };

        // Test error case detection
        assert!(corpus.is_error_case());
        assert_eq!(corpus.expected_error(), Some("ParseError"));
        assert_eq!(corpus.expected_error_message(), Some("Test error"));

        // Test tokens access
        assert!(corpus.tokens().is_none());
        corpus.processed_data = Some(ProcessedData::Tokens(vec![
            "test".to_string(),
            "content".to_string(),
        ]));
        assert_eq!(
            corpus.tokens(),
            Some(&vec!["test".to_string(), "content".to_string()])
        );
    }
}
