#[cfg(test)]
mod tests {
    use super::super::tutorial_export::{format_seconds, validate_filename};

    #[test]
    fn test_format_seconds() {
        assert_eq!(format_seconds(0.0), "00:00:00");
        assert_eq!(format_seconds(61.5), "00:01:01");
        assert_eq!(format_seconds(3661.0), "01:01:01");
        assert_eq!(format_seconds(61.9), "00:01:01");
        assert_eq!(format_seconds(-5.0), "00:00:00");
        assert_eq!(format_seconds(86400.0), "24:00:00");
    }

    #[test]
    fn test_validate_filename() {
        assert!(validate_filename("notes.md").is_ok());
        assert!(validate_filename("").is_err());
        assert!(validate_filename("notes.txt").is_err());
        assert!(validate_filename("path/notes.md").is_err());
        assert!(validate_filename("path\\notes.md").is_err());
    }
}
