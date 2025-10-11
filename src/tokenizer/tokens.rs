#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TokenType {
    // Indentation tokens
    Indent,
    Dedent,

    // Line structure tokens
    Newline,
    BlankLine,
    Text,

    // List/sequence markers
    SequenceMarker, // For 1., a), etc.
    Dash,           // For dash lists

    // Block markers
    PragmaMarker,  // ::
    VerbatimStart, // Line ending with single :
    VerbatimContent,
    VerbatimEnd,

    // Parameter/annotation tokens
    Identifier,
    String,
    Equals,
    Comma,
    Colon,

    // Inline formatting markers
    EmphasisMarker, // _
    StrongMarker,   // *
    CodeMarker,     // `
    MathMarker,     // #
    RefMarker,      // []
    SessionNumber,  // [#3.4]
    FootnoteNumber, // [21]
    Citation,       // [@key] or [@key, p. 45]

    // Block structure
    DefinitionMarker, // :: at end of line

    // Verbatim placeholders
    VerbatimPlaceholder, // §§VERBATIM_N§§

    // Special
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub value: Option<String>,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(token_type: TokenType, value: Option<String>, line: usize, column: usize) -> Self {
        Self {
            token_type,
            value,
            line,
            column,
        }
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.value {
            Some(value) => write!(
                f,
                "Token({:?}, {:?}, {}:{})",
                self.token_type, value, self.line, self.column
            ),
            None => write!(
                f,
                "Token({:?}, {}:{})",
                self.token_type, self.line, self.column
            ),
        }
    }
}
