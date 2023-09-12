use super::{TransferProgress, TransferStrategy};
use fs_extra::{
    dir::{
        move_dir_with_progress, CopyOptions as DirCopyOptions, TransitProcess as DirTransitProcess,
    },
    file::move_file_with_progress,
};
use std::{
    path::{Path, PathBuf},
    thread, time::Duration,
};

pub struct MoveStrategy;

impl TransferStrategy for MoveStrategy {
    fn transfer_dir<P: AsRef<std::path::Path>>(
        &mut self,
        source: P,
        destination: P,
        tx: std::sync::mpsc::Sender<super::TransferProgress>,
    ) {
        let mut options = DirCopyOptions::new();
        options.buffer_size = 8 * 1024 * 1024; // TODO: configurable buffer, default is 1MB
        let from = PathBuf::from(source.as_ref());
        let to = PathBuf::from(destination.as_ref());

        thread::spawn(move || {
            let progress_handler = |progress_info: DirTransitProcess| {
                if tx
                    .send(TransferProgress::DirTransfer(progress_info))
                    .is_ok()
                {}
                fs_extra::dir::TransitProcessResult::ContinueOrAbort
            };
            let _result = move_dir_with_progress(
                AsRef::<Path>::as_ref(&from),
                AsRef::<Path>::as_ref(&to),
                &options,
                progress_handler,
            );
        });
    }

    fn transfer_file<P: AsRef<std::path::Path>>(
        &mut self,
        source: P,
        destination: P,
        tx: std::sync::mpsc::Sender<super::TransferProgress>,
    ) {
        let mut options = fs_extra::file::CopyOptions::new();
        options.buffer_size = 8 * 1024 * 1024; // TODO: configurable buffer, default is 1MB
        let from = PathBuf::from(source.as_ref());
        let file_name = from.file_name().unwrap();
        let mut to = PathBuf::from(destination.as_ref());
        to.push(Path::new(file_name));

        thread::spawn(move || {
            let progress_handler = |progress_info: fs_extra::file::TransitProcess| {
                if tx
                    .send(TransferProgress::FileTransfer(progress_info))
                    .is_ok()
                {}
                thread::sleep(Duration::from_millis(500));
            };
            let _result = move_file_with_progress(
                AsRef::<Path>::as_ref(&from),
                AsRef::<Path>::as_ref(&to),
                &options,
                progress_handler,
            );
        });
    }
}
