// :: title :: Paragraph Element Specification
//
// Complete specification for paragraphs - the fundamental text blocks that contain inline content and form the basic unit of readable text in txxt documents.
//
// 1. Purpose
//
//     Paragraphs are the foundational building blocks for textual content in txxt. They contain inline text with formatting, create readable text flow, and serve as the default element type when no other block structure is detected. Paragraphs provide the semantic foundation for document content while supporting rich inline formatting including emphasis, code, references, and mathematical expressions.
//
// 2. Syntax
//
//     2.1. Basic Paragraph Form
//
//         Simple paragraph:
//             This is a basic paragraph containing plain text. It flows naturally and can span multiple lines within the same paragraph block.
//         :: txxt.core.spec.paragraph.valid.simple ::
//
//         Multiple paragraphs with blank line separation:
//             This is the first paragraph.
//
//             This is the second paragraph, separated by a blank line.
//         :: txxt.core.spec.paragraph.valid.multiple-with-blanks ::
//
//         Paragraph spanning multiple lines:
//             This paragraph begins on one line
//             and continues on the next line.
//             All lines at the same indentation level
//             belong to the same paragraph.
//         :: txxt.core.spec.paragraph.valid.multiline ::
//
//     2.4. Syntax Rules
//
//         Paragraph recognition:
//         - Lines contain text content (not whitespace-only)
//         - Lines at same indentation level continue the paragraph
//         - Blank line terminates the paragraph
//         - Serves as default element when no other pattern matches
//
//         Essential pattern:
//             <plain-text-line>+
//         :: pattern
//
//     2.5. Semantic Token List
//
//     In the semantic token list for the paragraph element we should see:
//         <plain-text-line><line-break>
//         <plain-text-line><line-break>

//     That is without special markers. Now let's make the simplest example:
//         <text-span>This is a paragraph<line-break>
//
// 3. AST Structure
//
// 	3.1 Expected Structure
// 		Post-parsing semantic representation:
//
// 		Paragraph AST:
// 			├── Paragraph
// 			│   ├── content: Vec<Inline>
// 			│   ├── annotations: Vec<Annotation>
// 			│   └── tokens: TokenSequence
// 		:: tree
//
// 	3.2. AST Assertion API
//
// 		Example API call for validating paragraph elements:
//
// 		```rust
// 		use tests::assertions::{assert_paragraph, ParagraphExpected};
//
// 		// Validate a paragraph element
// 		assert_paragraph(&element, ParagraphExpected {
// 			text_contains: Some("This is a paragraph"),
// 			has_formatting: Some(false),
// 			annotation_count: Some(0),
// 			..Default::default()
// 		});
// 		```
//
// 4. Text Input Generation:
//
// 	For this element we will use:
// 		This is a simple paragraph with plain text content.
// 	:: example1
//
// 	And with formatting:
// 		This paragraph contains *bold text* and _italic text_.
// 	:: example2
//
//     User corpora tool to load that string and return the Semantic Token List for that string.
