// :: title :: Session Element Specification
//
// Complete specification for sessions - the hierarchical organizational units that structure txxt documents into navigable sections.
//
// 1. Purpose
//
//     Sessions are the primary organizational element in txxt documents, providing hierarchical structure similar to chapters, sections, and subsections. They enable document navigation, content organization, and automated table of contents generation. Sessions can be numbered or unnumbered and support arbitrary nesting depth with flexible numbering schemes.
//
// 2. Syntax
//
//     2.1. Basic Session Forms
//
//         Unnumbered session:
//             Introduction
//
//                 Content of the introduction section goes here.
//                 Multiple paragraphs are supported.
//         :: txxt.core.spec.session.valid.unnumbered-basic ::
//
//         Numbered session (numbering is stylistic):
//             1. Methodology
//
//                 Detailed methodology content follows.
//                 Includes all relevant procedures.
//         :: txxt.core.spec.session.valid.numbered-basic ::
//
//     2.2. Recognition Rules
//
//         Session identification requirements:
//         - Preceded by blank line (or start of document)
//         - Followed by indented content (+1 indentation level)
//         - Without indented content → Parsed as paragraph
//         - Session titles cannot start with dash (but other sequence markers are ok)
//
//         Essential pattern:
//             <blank-line>
//             <text-line>
//             <blank-line>
//             <indent>
//             <content-blocks>
//         :: pattern
//
//         For detailed disambiguation rules between sessions, lists, and paragraphs, see @12-complex-sessions.txxt.
//
//     2.3. Semantic Token List
//
//     In the semantic token list for the session element we should see:
//         <blank-line><line-break>
//         <text-line><line-break>
//         <blank-line><line-break>
//         <indent>
//         <plain-text-line><line-break>
//     :: semantic-token-list
//     That is with blank line separation and indented content. Now let's make the simplest example:
//         <blank-line><line-break>
//         <text-span>Introduction<line-break>
//         <blank-line><line-break>
//         <indent><text-span>Content here<line-break>
//     :: semantic-token-list
//
// 3. AST Structure
//
// 	3.1 Expected Structure
// 		Post-parsing semantic representation:
//
// 		Session AST:
// 			├── Session
// 			│   ├── title: SessionTitle
// 			│   │   ├── content: Vec<Inline>
// 			│   │   ├── numbering: Option<SessionNumbering>
// 			│   │   └── tokens: TokenSequence
// 			│   ├── content: Container
// 			│   │   └── content: Vec<Block>
// 			│   ├── annotations: Vec<Annotation>
// 			│   └── tokens: TokenSequence
// 		:: tree
//
// 	3.2. AST Assertion API
//
// 		Example API call for validating session elements:
//
// 		```rust
// 		use tests::assertions::{assert_session, SessionExpected};
//
// 		// Validate a session element
// 		assert_session(&element, SessionExpected {
// 			title_contains: Some("Introduction"),
// 			is_numbered: Some(false),
// 			child_count: Some(1),
// 			has_subsession: Some(false),
// 			..Default::default()
// 		});
// 		```
//
// 4. Text Input Generation:
//
// 	For this element we will use:
// 		Introduction
//
// 		    This is the content of the introduction section.
// 	:: example1
//
// 	And with numbering (stylistic):
// 		1. Methodology
//
// 		    Detailed methodology content follows.
// 	:: example2
//
//     User corpora tool to load that string and return the Semantic Token List for that string.
