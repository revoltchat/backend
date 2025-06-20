pub fn transform_optional_string(s: Option<&str>) -> Option<String> {
    s.map(|f| f.to_string())
}
