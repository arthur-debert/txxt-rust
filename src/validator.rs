use crate::Annotation;

pub fn validate_annotation(_annotation: &Annotation) -> Result<bool, Box<dyn std::error::Error>> {
    Ok(true)
}
