# Important txxt code poinsts

## Tooling

- `divernv allow` or `source .envrc` for path and tool access
- cargo test , cargo build, cargo run

## NON NEGOTIABLES

- No skipping tests, by passim with --no-verify
- No broken tests

### Testing

- Use of rs tests, including multiple inputs and edge cases via named cases.
- use of proptest for property based testing
- Small strings (like tokens) can be test embeded, but larger txxt fragments
  must used the vetoed txxt files in the txxt-documents-clean fir.

### Documentation 

- No use of writing a doc that repeats the code in natural language.
- Use of doc comments to explain the why and how of the code, reference use
  cases and non obvious needs, document design decisions.
 
