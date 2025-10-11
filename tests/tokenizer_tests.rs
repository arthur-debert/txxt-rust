use std::fs;
use std::path::Path;
use txxt::tokenizer::{tokenize, TokenType};

/// Test fixture for loading txxt files and their expected token XML files
pub struct TxxtTestFixture {
    pub name: String,
    pub txxt_content: String,
    pub expected_tokens: Vec<ExpectedToken>,
}

#[derive(Debug, PartialEq)]
pub struct ExpectedToken {
    pub token_type: String,
    pub line: usize,
    pub column: usize,
    pub value: String,
}

impl TxxtTestFixture {
    /// Load a test fixture from the txxt-documents-clean directory
    pub fn load(relative_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let base_path = Path::new("txxt-documents-clean").join(relative_path);
        let txxt_path = base_path.with_extension("txxt");
        let tokens_path = format!("{}.tokens.xml", txxt_path.to_string_lossy());

        let txxt_content = fs::read_to_string(&txxt_path)
            .map_err(|e| format!("Failed to read {}: {}", txxt_path.display(), e))?;

        let tokens_xml = fs::read_to_string(&tokens_path)
            .map_err(|e| format!("Failed to read {}: {}", tokens_path, e))?;

        let expected_tokens = parse_expected_tokens(&tokens_xml)?;

        Ok(TxxtTestFixture {
            name: relative_path.to_string(),
            txxt_content,
            expected_tokens,
        })
    }

    /// Run the tokenizer and compare against expected tokens
    pub fn run_test(&self) -> Result<(), String> {
        let actual_tokens = tokenize(&self.txxt_content);

        // Convert our tokens to the expected format for comparison
        let actual_expected: Vec<ExpectedToken> = actual_tokens
            .iter()
            .map(|t| {
                let token_type_str = format!("{:?}", t.token_type);
                // Convert from PascalCase to SNAKE_CASE
                let snake_case = token_type_str
                    .chars()
                    .enumerate()
                    .map(|(i, c)| {
                        if i > 0 && c.is_uppercase() {
                            format!("_{}", c)
                        } else {
                            c.to_string()
                        }
                    })
                    .collect::<String>()
                    .to_uppercase();

                ExpectedToken {
                    token_type: snake_case,
                    line: t.line,
                    column: t.column,
                    value: t.value.clone().unwrap_or_default(),
                }
            })
            .collect();

        if actual_expected.len() != self.expected_tokens.len() {
            return Err(format!(
                "Token count mismatch for {}: expected {}, got {}",
                self.name,
                self.expected_tokens.len(),
                actual_expected.len()
            ));
        }

        for (i, (actual, expected)) in actual_expected
            .iter()
            .zip(self.expected_tokens.iter())
            .enumerate()
        {
            if actual != expected {
                return Err(format!(
                    "Token mismatch at position {} for {}:\nExpected: {:?}\nActual: {:?}",
                    i, self.name, expected, actual
                ));
            }
        }

        Ok(())
    }
}

/// Parse the expected tokens from the XML format
fn parse_expected_tokens(
    xml_content: &str,
) -> Result<Vec<ExpectedToken>, Box<dyn std::error::Error>> {
    let mut tokens = Vec::new();

    // Simple XML parsing - look for <item> blocks (with DOTALL flag)
    let item_pattern = regex::Regex::new(r"(?s)<item>(.*?)</item>")?;
    let type_pattern = regex::Regex::new(r"<type>(.*?)</type>")?;
    let line_pattern = regex::Regex::new(r"<line>(\d+)</line>")?;
    let column_pattern = regex::Regex::new(r"<column>(\d+)</column>")?;
    let value_pattern = regex::Regex::new(r"(?s)<value>(.*?)</value>")?;

    for item_match in item_pattern.find_iter(xml_content) {
        let item_content = item_match.as_str();

        let token_type = type_pattern
            .find(item_content)
            .and_then(|m| type_pattern.captures(m.as_str()))
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();

        let line = line_pattern
            .find(item_content)
            .and_then(|m| line_pattern.captures(m.as_str()))
            .and_then(|caps| caps.get(1))
            .and_then(|m| m.as_str().parse().ok())
            .unwrap_or(0);

        let column = column_pattern
            .find(item_content)
            .and_then(|m| column_pattern.captures(m.as_str()))
            .and_then(|caps| caps.get(1))
            .and_then(|m| m.as_str().parse().ok())
            .unwrap_or(0);

        let value = value_pattern
            .find(item_content)
            .and_then(|m| value_pattern.captures(m.as_str()))
            .and_then(|caps| caps.get(1))
            .map(|m| decode_xml_entities(m.as_str()))
            .unwrap_or_default();

        tokens.push(ExpectedToken {
            token_type,
            line,
            column,
            value,
        });
    }

    Ok(tokens)
}

/// Decode XML entities like &quot; &amp; etc.
fn decode_xml_entities(text: &str) -> String {
    text.replace("&quot;", "\"")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&#x27;", "'")
}

#[test]
fn test_nano_spec() {
    let fixture = TxxtTestFixture::load("specs/nano").expect("Failed to load nano test fixture");
    fixture.run_test().expect("Nano test failed");
}

#[test]
fn test_micro_spec() {
    let fixture = TxxtTestFixture::load("specs/micro").expect("Failed to load micro test fixture");
    fixture.run_test().expect("Micro test failed");
}

#[test]
fn test_simple_ensemble() {
    let fixture =
        TxxtTestFixture::load("ensambles/simple").expect("Failed to load simple ensemble fixture");
    fixture.run_test().expect("Simple ensemble test failed");
}

#[test]
fn test_annotation_simple() {
    let fixture = TxxtTestFixture::load("elements/annotations/annotation-simple")
        .expect("Failed to load annotation-simple fixture");
    fixture.run_test().expect("Annotation simple test failed");
}

#[test]
fn test_basic_tokenization() {
    let text = "This is a simple paragraph.";
    let tokens = tokenize(text);

    assert!(!tokens.is_empty());
    assert_eq!(tokens[0].token_type, TokenType::Text);
    assert_eq!(
        tokens[0].value,
        Some("This is a simple paragraph.".to_string())
    );
}

#[test]
fn test_list_tokenization() {
    let text = "1. First item\n2. Second item";
    let tokens = tokenize(text);

    // Should contain sequence markers
    let sequence_markers: Vec<_> = tokens
        .iter()
        .filter(|t| t.token_type == TokenType::SequenceMarker)
        .collect();
    assert_eq!(sequence_markers.len(), 2);
}

#[test]
fn test_verbatim_block() {
    let text = "Code example:\n    console.log('hello');\n(javascript)";
    let tokens = tokenize(text);

    let verbatim_tokens: Vec<_> = tokens
        .iter()
        .filter(|t| {
            matches!(
                t.token_type,
                TokenType::VerbatimStart | TokenType::VerbatimContent | TokenType::VerbatimEnd
            )
        })
        .collect();
    assert!(!verbatim_tokens.is_empty());
}

#[test]
fn test_pragma_annotation() {
    let text = ":: title :: My Document";
    let tokens = tokenize(text);

    let pragma_tokens: Vec<_> = tokens
        .iter()
        .filter(|t| t.token_type == TokenType::PragmaMarker)
        .collect();
    assert_eq!(pragma_tokens.len(), 2);
}
