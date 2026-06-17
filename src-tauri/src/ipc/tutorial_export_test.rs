#[cfg(test)]
mod tests {
    use super::super::tutorial_export::format_seconds;

    #[test]
    fn test_format_seconds() {
        assert_eq!(format_seconds(0.0), "00:00:00");
        assert_eq!(format_seconds(61.5), "00:01:01");
        assert_eq!(format_seconds(3661.0), "01:01:01");
    }
}
