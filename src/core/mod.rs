pub mod list_dir;

pub fn calculate_progress_percentage(progress: &fs_extra::dir::TransitProcess) -> (u64, u64) {
    let total_percent;
    if progress.copied_bytes != 0 && progress.total_bytes != 0 {
        total_percent = progress.copied_bytes as f64 / (progress.total_bytes as f64) * 100.0;
    } else {
        total_percent = 0.0;
    }

    let partial_percent;
    if progress.file_bytes_copied != 0 && progress.file_total_bytes != 0 {
        partial_percent =
            (progress.file_bytes_copied as f64 / progress.file_total_bytes as f64) * 100.0;
    } else {
        partial_percent = 0.0;
    }
    return (total_percent as u64, partial_percent as u64);
}

#[cfg(test)]
mod tests {
    use super::*;
    use fs_extra::dir::{TransitProcess, TransitState};

    #[test]
    fn test_calculate_progress_percentage() {
        let progress = TransitProcess {
            copied_bytes: 1024,
            total_bytes: 4086,
            file_bytes_copied: 128,
            file_total_bytes: 256,
            file_name: String::new(),
            state: TransitState::Normal,
        };

        let (total_percent, partial_percent) = calculate_progress_percentage(&progress);

        assert_eq!(total_percent, 25);
        assert_eq!(partial_percent, 50);
    }
}
