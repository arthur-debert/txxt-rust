use std::path::PathBuf;
use txxt::{Annotation, Txxt};

#[test]
fn test_txxt_creation() {
    let path = PathBuf::from("test.txxt");
    let txxt_instance = Txxt::new(path.clone());

    assert_eq!(txxt_instance.path, path);
    assert!(txxt_instance.annotations.is_empty());
}

#[test]
fn test_annotation_creation() {
    let annotation = Annotation {
        path: PathBuf::from("test.txt"),
        text: "Test annotation".to_string(),
        source_file: PathBuf::from("test.txxt"),
    };

    assert_eq!(annotation.text, "Test annotation");
}
