use crate::Annotation;
use std::path::Path;

pub fn parse_file(_path: &Path) -> Result<Vec<Annotation>, Box<dyn std::error::Error>> {
    Ok(vec![])
}

pub fn write_file(
    _path: &Path,
    _annotations: &[Annotation],
) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
