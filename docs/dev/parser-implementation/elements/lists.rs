// :: title :: List Element Specification
//
// Complete specification for lists - ordered and unordered collections of items with sophisticated styling and nesting support.
//
// 1. Purpose
//
//     Lists provide structured collections of related items with support for complex numbering schemes, nesting, and flexible authoring patterns. Lists in txxt are designed for technical documentation needs, supporting everything from simple bullet points to complex hierarchical outlines with mixed numbering styles.
//
// 2. Syntax
//
//     2.1. Basic List Styles
//
//         Plain style list:
//             - First item
//             - Second item
//             - Third item
//         :: txxt.core.spec.list.valid.plain-flat ::
//
//         Numerical style list:
//             1. First item
//             2. Second item
//             3. Third item
//         :: txxt.core.spec.list.valid.numerical-flat ::
//
//         Alphabetical style list:
//             a. First item
//             b. Second item
//             c. Third item
//         :: txxt.core.spec.list.valid.alphabetical-flat ::
//
//         Roman style list:
//             i. First item
//             ii. Second item
//             iii. Third item
//         :: txxt.core.spec.list.valid.roman-flat ::
//
//     2.2. Recognition Rules
//
//         List identification requirements:
//         - Must have sequence markers (dash, number, letter, roman)
//         - Requires multiple items (single items are parsed as paragraphs)
//         - Items cannot have blank lines between them
//         - Items can have nested content (indented)
//
//         Essential pattern:
//             <sequence-text-line>
//             <sequence-text-line>
//             <sequence-text-line>+
//         :: pattern
//
//         Critical Rule: Single items at the top level are parsed as paragraphs, not lists. Lists require multiple items at the root level to disambiguate from dialog and ensure clear intent.
//
//         For detailed disambiguation rules between sessions, lists, and paragraphs, see @12-complex-sessions.txxt.
//
//     2.3. Semantic Token List
//
//     In the semantic token list for the list element we should see:
//         <sequence-marker><text-span><line-break>
//         <sequence-marker><text-span><line-break>
//         <sequence-marker><text-span><line-break>
//     :: semantic-token-list
//     That is with sequence markers and no blank lines between items. Now let's make the simplest example:
//         <sequence-marker>-<text-span>First item<line-break>
//         <sequence-marker>-<text-span>Second item<line-break>
//     :: semantic-token-list
//
// 3. AST Structure
//
// 	3.1 Expected Structure
// 		Post-parsing semantic representation:
//
// 		List AST:
// 			├── List
// 			│   ├── decoration_type: ListDecorationType
// 			│   │   ├── style: NumberingStyle
// 			│   │   └── form: NumberingForm
// 			│   ├── items: Vec<ListItem>
// 			│   │   ├── content: Vec<Inline>
// 			│   │   ├── container: Option<Container>
// 			│   │   └── tokens: TokenSequence
// 			│   ├── annotations: Vec<Annotation>
// 			│   └── tokens: TokenSequence
// 		:: tree
//
// 	3.2. AST Assertion API
//
// 		Example API call for validating list elements:
//
// 		```rust
// 		use tests::assertions::{assert_list, ListExpected};
//
// 		// Validate a list element
// 		assert_list(&element, ListExpected {
// 			style: Some(NumberingStyle::Plain),
// 			item_count: Some(2),
// 			item_text: Some(vec!["First item", "Second item"]),
// 			has_nested: Some(vec![false, false]),
// 			..Default::default()
// 		});
// 		```
//
// 4. Text Input Generation:
//
// 	For this element we will use:
// 		- First item
// 		- Second item
// 	:: example1
//
// 	And with numbering:
// 		1. First item
// 		2. Second item
// 	:: example2
//
//     User corpora tool to load that string and return the Semantic Token List for that string.
