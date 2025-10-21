#[cfg(test)]
mod tests {
    use txxt::syntax::tokenize;

    #[test]
    fn debug_definition_vs_annotation_markers() {
        // Test cases that should NOT produce definition markers
        let failing_cases = vec![":: label ::", ":: title ::", "::", ":::", "::::"];

        for input in failing_cases {
            println!("Analyzing input: '{}'", input);
            let tokens = tokenize(input);

            for (i, token) in tokens.iter().enumerate() {
                println!("  {}: {:?}", i, token);
            }
            println!();
        }

        // Test cases that SHOULD produce definition markers
        let passing_cases = vec!["term ::", "definition ::"];

        for input in passing_cases {
            println!("Analyzing input: '{}'", input);
            let tokens = tokenize(input);

            for (i, token) in tokens.iter().enumerate() {
                println!("  {}: {:?}", i, token);
            }
            println!();
        }
    }
}
