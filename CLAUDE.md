# Rules for txxt development

## Tooling

- `divernv allow` or `source .envrc` for path and tool access
- cargo test , cargo build, cargo run

## NON NEGOTIABLES

- No skipping tests, by passim with --no-verify
- No broken tests

### Testing

- Use of rs tests, including multiple inputs and edge cases via named cases.
- use of proptest for property based testing
- Small strings (like tokens) can be test embedded, but larger txxt fragments
  must used the vetoed txxt files in the txxt-documents-clean fir.

### Documentation

- No use of writing a doc that repeats the code in natural language.
- Use of doc comments to explain the why and how of the code, reference use
  cases and non obvious needs, document design decisions.
  Liberal or module level docs for context, see src/ast.rs for an example.

### Workflow

- Use of branches for features and fixes, no direct commits to main.
- Use of PRs for code review, no direct merges to main.
- Atomic commits with clear messages, no large or vague commits. If your task
  has several parts, break it into multiple commits.

### Backwards Compatibility: NEVER

- This is unreleased software, we must break things to make progress.
- Compatibility layers, adapters and deprecated code are not allowed ,
   updating callers and tests is a big part of the works.
   Small changes that cause large number of test changes are to be fixed, but a
   smell sign that tests are too brittle or need setup infrastructure.

### Text Formatting and Style / Tone

- This is the repo for a plain text format, txxt, so documentation and text
  must be in txxt format. No markdown.
- The tour [docs/tour.txxt] and vs markdown [docs/not-markdown.txxt] are quick
a howtos for txxt, specially how not to use markdown.

### txxt format is finally in final draft + fully detailed syntax

After a few rounds experemitentation of coding and spec refactor we have finnaly commited to a final spec draft.
Now, we have systematically design the full syntax, with the grammar and tokens, and organized the information [docs/specs]

- The core tipology of elements and nodes: docs/specs/core/terminology.txxt
- The format's syntax: docs/specs/core/syntax.txxt
- docs/specs/core/elements/
- The core reusable blocks (docs/specs/elements/container.txxt docs/specs/elements/labels.txxt docs/specs/elements/parameters.txxt)
- Then docs/specs/elements/ `<elements> for the work you'll be doing
- This is the authorative information on txxt, and these will win every other source
