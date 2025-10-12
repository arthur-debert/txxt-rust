//! Testing framework verification
//!
//! Verifies that rstest and proptest are working correctly

use proptest::prelude::*;
use rstest::rstest;

#[rstest]
#[case("hello", 5)]
#[case("world", 5)]
#[case("", 0)]
fn test_rstest_string_length(#[case] input: &str, #[case] expected: usize) {
    assert_eq!(input.len(), expected);
}

proptest! {
    #[test]
    fn test_proptest_string_reverse(s in ".*") {
        let reversed: String = s.chars().rev().collect();
        let double_reversed: String = reversed.chars().rev().collect();
        prop_assert_eq!(s, double_reversed);
    }
}

#[test]
fn test_basic_functionality() {
    assert_eq!(2 + 2, 4);
}
