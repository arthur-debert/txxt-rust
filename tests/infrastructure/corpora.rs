use std::collections::HashMap;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

// Use the unified API for processing
use txxt::api::{format_output_unified, process_unified, Format, Output, Stage};

/// Processing stages for test corpora (re-exported from unified API).
pub use txxt::api::Stage as ProcessingStage;

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
    /// Get the raw output from processing (using unified API).
    #[allow(dead_code)] // Will be used by parser tests
    pub fn output(&self) -> Option<&Output> {
        match &self.processed_data {
            Some(ProcessedData::Output(output)) => Some(output),
            _ => None,
        }
    }

    /// Get the processed data formatted as JSON.
    #[allow(dead_code)] // Will be used by parser tests
    pub fn as_json(&self) -> Option<String> {
        match &self.processed_data {
            Some(ProcessedData::Output(output)) => {
                format_output_unified(output, Format::Json, Some(&self.name)).ok()
            }
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

/// Processed data from pipeline stages (using unified API).
#[derive(Debug, Clone)]
pub enum ProcessedData {
    /// Processed output from unified API
    Output(Output),
}

// Manual PartialEq since Output doesn't implement it
impl PartialEq for ProcessedData {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (ProcessedData::Output(_), ProcessedData::Output(_))
        )
    }
}

/// Spec-driven test corpus loader for txxt parser testing.
///
/// **Quick Usage:**
/// ```rust
/// // Load raw text: TxxtCorpora::load("txxt.core.spec.paragraph.valid.simple")
/// // Load pre-tokenized: TxxtCorpora::load_with_processing("name", ProcessingStage::ScannerTokens)
/// // Load documents: TxxtCorpora::load_document("01-two-paragraphs")
/// ```
///
/// Provides two loading modes:
/// 1. **Fragments**: Isolated element samples from spec verbatim blocks
/// 2. **Documents**: Complete files from ensembles directory
///
/// # Fragment Loading (Isolated Elements)
///
/// Load tagged samples from specification documents:
///
/// ```rust
/// use tests::corpora::TxxtCorpora;
///
/// // Load element fragment by label
/// let corpus = TxxtCorpora::load("txxt.core.spec.paragraph.valid.simple")?;
/// assert!(corpus.source_text.contains("paragraph"));
///
/// // Test invalid cases with error parameters
/// let corpus = TxxtCorpora::load("txxt.core.spec.definition.invalid.empty-term")?;
/// assert!(corpus.is_error_case());
/// assert_eq!(corpus.expected_error(), Some("EmptyTerm"));
/// ```
///
/// Fragments are extracted from verbatim blocks in `docs/specs/elements/*.txxt` with labels
/// like `:: txxt.core.spec.element.validity.testcase ::`.
///
/// # Document Loading (Full Documents)
///
/// Load complete documents for integration testing:
///
/// ```rust
/// use tests::corpora::TxxtCorpora;
///
/// // Load by name or number prefix
/// let corpus = TxxtCorpora::load_document("01-two-paragraphs")?;
/// // Or shorter: TxxtCorpora::load_document("01")?;
///
/// // Load all documents in order
/// let docs = TxxtCorpora::load_all_documents()?;
/// for doc in docs {
///     let result = parse(&doc.source_text);
///     assert!(result.is_ok());
/// }
/// ```
///
/// Documents are in `docs/specs/ensembles/` and progress from simple (01) to
/// comprehensive (11), enabling progressive parser validation.
///
/// # Processing Stages
///
/// Both modes support pipeline stages: `Raw` (default), `Tokens`, `HighLevelTokens`, `BlockedTokens`,
/// `ParsedAst`, `FullDocument`.
///
/// ```rust
/// use tests::corpora::{TxxtCorpora, ProcessingStage};
///
/// // Fragment with processing
/// let corpus = TxxtCorpora::load_with_processing(
///     "txxt.core.spec.list.valid.plain-flat",
///     ProcessingStage::HighLevelTokens
/// )?;
///
/// // Document with processing
/// let corpus = TxxtCorpora::load_document_with_processing(
///     "11-full-document",
///     ProcessingStage::AstFull
/// )?;
/// ```
///
/// # Testing Strategy
///
/// - **Fragments**: Validate isolated element parsing, good for unit tests
/// - **Documents**: Validate complete parsing, good for integration tests
/// - **Progressive**: Test docs 01→11 to isolate parser capabilities by complexity
///
/// See `tests/ensemble_documents_example.rs` for comprehensive usage examples.
pub struct TxxtCorpora;

impl TxxtCorpora {
    /// Load a specific test corpus by name with default (Raw) processing.
    ///
    /// The name should follow the pattern `txxt.core.spec.*` as defined in the
    /// specification documents under `docs/specs/`.
    #[allow(dead_code)] // Will be used by parser tests
    pub fn load(name: &str) -> Result<Corpus, CorpusError> {
        Self::load_with_processing(name, ProcessingStage::ScannerTokens)
    }

    /// Load a specific test corpus by name with specified processing stage.
    ///
    /// The name should follow the pattern `txxt.core.spec.*` as defined in the
    /// specification documents under `docs/specs/`.
    ///
    /// # Arguments
    /// * `name` - The corpus identifier (e.g., "txxt.core.spec.paragraph.valid.simple")
    /// * `processing` - The processing stage to apply to the corpus
    #[allow(dead_code)] // Will be used by parser tests
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
    #[allow(dead_code)] // Will be used by parser tests
    pub fn load_all() -> Result<Vec<Corpus>, CorpusError> {
        Self::load_all_with_processing(ProcessingStage::ScannerTokens)
    }

    /// Extract all corpora from the specification documents with specified processing stage.
    #[allow(dead_code)] // Will be used by parser tests
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

    /// Load a complete document file from the ensembles directory with default (Raw) processing.
    ///
    /// This method loads entire txxt documents for integration testing, as opposed to
    /// extracting individual test cases from verbatim blocks.
    ///
    /// # Document Naming Convention
    ///
    /// Ensemble documents follow the pattern: `NN-descriptive-name.txxt`
    /// - Located in `docs/specs/ensembles/`
    /// - NN: Two-digit sequence number (01, 02, etc.)
    /// - descriptive-name: Kebab-case description
    ///
    /// # Arguments
    ///
    /// * `name` - The document name without extension (e.g., "01-two-paragraphs" or just "01")
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tests::corpora::TxxtCorpora;
    ///
    /// // Load the simplest ensemble document
    /// let corpus = TxxtCorpora::load_document("01-two-paragraphs").unwrap();
    /// assert!(corpus.source_text.contains("first paragraph"));
    ///
    /// // Can also use just the number
    /// let corpus = TxxtCorpora::load_document("01").unwrap();
    /// ```
    ///
    /// # Returns
    ///
    /// Returns a `Corpus` containing the entire document text with:
    /// - `name`: The filename without extension
    /// - `source_text`: Complete document content
    /// - `parameters`: Empty HashMap (documents don't have parameters)
    /// - `processing_stage`: Set to Raw by default
    ///
    /// # Errors
    ///
    /// Returns `CorpusError` if:
    /// - Ensembles directory doesn't exist
    /// - Document file not found
    /// - File cannot be read
    #[allow(dead_code)] // Will be used by parser tests
    pub fn load_document(name: &str) -> Result<Corpus, CorpusError> {
        Self::load_document_with_processing(name, ProcessingStage::ScannerTokens)
    }

    /// Load a complete document file with specified processing stage.
    ///
    /// This is the core method for loading ensemble documents with control over
    /// processing stage for integration with different pipeline stages.
    ///
    /// # Arguments
    ///
    /// * `name` - The document name (with or without extension)
    /// * `processing` - The processing stage to apply
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tests::corpora::{TxxtCorpora, ProcessingStage};
    ///
    /// // Load document for tokenization testing
    /// let corpus = TxxtCorpora::load_document_with_processing(
    ///     "11-full-document",
    ///     ProcessingStage::ScannerTokens
    /// ).unwrap();
    ///
    /// // Load document for full AST parsing
    /// let corpus = TxxtCorpora::load_document_with_processing(
    ///     "09-nested-complex",
    ///     ProcessingStage::AstFull
    /// ).unwrap();
    /// ```
    #[allow(dead_code)] // Will be used by parser tests
    pub fn load_document_with_processing(
        name: &str,
        processing: ProcessingStage,
    ) -> Result<Corpus, CorpusError> {
        let ensembles_dir = Path::new("docs/specs/ensembles");
        if !ensembles_dir.exists() {
            return Err(CorpusError::EnsemblesDirectoryNotFound);
        }

        // Try to find the file - support both with and without extension
        let name_without_ext = name.trim_end_matches(".txxt");

        // Try matching by prefix (e.g., "01" matches "01-two-paragraphs.txxt")
        for entry in WalkDir::new(ensembles_dir).max_depth(1) {
            let entry = entry.map_err(|e| CorpusError::FileSystemError(e.to_string()))?;

            if !entry.file_type().is_file() {
                continue;
            }

            let file_path = entry.path();
            if file_path.extension() != Some("txxt".as_ref()) {
                continue;
            }

            if let Some(file_name) = file_path.file_stem() {
                let file_name_str = file_name.to_string_lossy();

                // Match if:
                // 1. Exact match (e.g., "01-two-paragraphs" == "01-two-paragraphs")
                // 2. Prefix match (e.g., "01" matches "01-two-paragraphs")
                if file_name_str == name_without_ext
                    || file_name_str.starts_with(&format!("{}-", name_without_ext))
                {
                    let content = fs::read_to_string(file_path).map_err(|e| {
                        CorpusError::FileReadError(
                            file_path.to_string_lossy().to_string(),
                            e.to_string(),
                        )
                    })?;

                    let mut corpus = Corpus {
                        name: file_name_str.to_string(),
                        source_text: content,
                        parameters: HashMap::new(), // Documents don't have parameters
                        processing_stage: ProcessingStage::ScannerTokens,
                        processed_data: None,
                    };

                    Self::apply_processing(&mut corpus, processing)?;
                    return Ok(corpus);
                }
            }
        }

        Err(CorpusError::DocumentNotFound(name.to_string()))
    }

    /// Load all ensemble documents from the ensembles directory.
    ///
    /// This method loads all complete document files for comprehensive testing
    /// of the full document parsing pipeline.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tests::corpora::TxxtCorpora;
    ///
    /// // Load all ensemble documents
    /// let documents = TxxtCorpora::load_all_documents().unwrap();
    /// println!("Found {} ensemble documents", documents.len());
    ///
    /// for doc in &documents {
    ///     println!("Document: {}", doc.name);
    /// }
    /// ```
    ///
    /// # Returns
    ///
    /// Returns a `Vec<Corpus>` containing all ensemble documents, ordered by filename.
    #[allow(dead_code)] // Will be used by parser tests
    pub fn load_all_documents() -> Result<Vec<Corpus>, CorpusError> {
        Self::load_all_documents_with_processing(ProcessingStage::ScannerTokens)
    }

    /// Load all ensemble documents with specified processing stage.
    ///
    /// # Arguments
    ///
    /// * `processing` - The processing stage to apply to all documents
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tests::corpora::{TxxtCorpora, ProcessingStage};
    ///
    /// // Load all documents for AST validation
    /// let documents = TxxtCorpora::load_all_documents_with_processing(
    ///     ProcessingStage::AstFull
    /// ).unwrap();
    /// ```
    #[allow(dead_code)] // Will be used by parser tests
    pub fn load_all_documents_with_processing(
        processing: ProcessingStage,
    ) -> Result<Vec<Corpus>, CorpusError> {
        let ensembles_dir = Path::new("docs/specs/ensembles");
        if !ensembles_dir.exists() {
            return Err(CorpusError::EnsemblesDirectoryNotFound);
        }

        let mut documents = Vec::new();
        let mut entries: Vec<_> = WalkDir::new(ensembles_dir)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| e.path().extension() == Some("txxt".as_ref()))
            .collect();

        // Sort by filename for consistent ordering
        entries.sort_by_key(|e| e.path().to_path_buf());

        for entry in entries {
            let file_path = entry.path();
            let file_name = file_path
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string());

            let content = fs::read_to_string(file_path).map_err(|e| {
                CorpusError::FileReadError(file_path.to_string_lossy().to_string(), e.to_string())
            })?;

            let mut corpus = Corpus {
                name: file_name,
                source_text: content,
                parameters: HashMap::new(),
                processing_stage: ProcessingStage::ScannerTokens,
                processed_data: None,
            };

            Self::apply_processing(&mut corpus, processing)?;
            documents.push(corpus);
        }

        Ok(documents)
    }

    /// Apply processing stage to a corpus using the unified API.
    ///
    /// This delegates all processing to the main API, keeping corpora focused
    /// on loading test data from specification documents.
    fn apply_processing(corpus: &mut Corpus, processing: Stage) -> Result<(), CorpusError> {
        corpus.processing_stage = processing;

        // Use the unified API to process the source text
        let output = process_unified(&corpus.source_text, processing, Some(corpus.name.clone()))
            .map_err(|e| CorpusError::ProcessingError(e.to_string()))?;

        corpus.processed_data = Some(ProcessedData::Output(output));
        Ok(())
    }

    /// Extract a specific corpus from a file.
    #[allow(dead_code)] // Will be used by parser tests
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
    #[allow(dead_code)] // Will be used by parser tests
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
#[allow(dead_code)] // Will be used by parser tests
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
            processing_stage: Stage::ScannerTokens,
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
#[allow(dead_code)] // Variants will be used by parser tests
pub enum CorpusError {
    SpecsDirectoryNotFound,
    EnsemblesDirectoryNotFound,
    FileSystemError(String),
    FileReadError(String, String),
    CorpusNotFound(String),
    DocumentNotFound(String),
    TitleLineNotFound(String),
    ProcessingError(String),
}

impl std::fmt::Display for CorpusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CorpusError::SpecsDirectoryNotFound => {
                write!(f, "Specifications directory 'docs/specs' not found")
            }
            CorpusError::EnsemblesDirectoryNotFound => {
                write!(f, "Ensembles directory 'docs/specs/ensembles' not found")
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
            CorpusError::DocumentNotFound(name) => {
                write!(
                    f,
                    "Ensemble document '{}' not found in 'docs/specs/ensembles'",
                    name
                )
            }
            CorpusError::TitleLineNotFound(name) => {
                write!(f, "Title line not found for corpus '{}'", name)
            }
            CorpusError::ProcessingError(msg) => {
                write!(f, "Processing error: {}", msg)
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
    fn test_corpus_convenience_methods() {
        use super::*;

        let corpus = Corpus {
            name: "test".to_string(),
            source_text: "test content".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert("error".to_string(), "ParseError".to_string());
                params.insert("message".to_string(), "Test error".to_string());
                params
            },
            processing_stage: Stage::ScannerTokens,
            processed_data: None,
        };

        // Test error case detection
        assert!(corpus.is_error_case());
        assert_eq!(corpus.expected_error(), Some("ParseError"));
        assert_eq!(corpus.expected_error_message(), Some("Test error"));

        // Test output access before processing
        assert!(corpus.output().is_none());
        assert!(corpus.as_json().is_none());
    }
}
