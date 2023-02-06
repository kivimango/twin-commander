pub mod list_dir;

pub fn calculate_progress_percentage(partial_bytes: u64, total_bytes: u64) -> u64 {
    if partial_bytes != 0 && total_bytes != 0 {
        ((partial_bytes as f64 / total_bytes as f64) * 100.0) as u64
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_progress_percentage() {
        let copied_bytes = 1024;
        let total_bytes = 4086;
        let percentage = calculate_progress_percentage(copied_bytes, total_bytes);
        assert_eq!(percentage, 25);
    }

    #[test]
    fn test_calculate_progress_percentage_with_zero_should_not_panic() {
        let copied_bytes = 0;
        let total_bytes = 0;
        let percentage = calculate_progress_percentage(copied_bytes, total_bytes);
        assert_eq!(percentage, 0);
    }
}
