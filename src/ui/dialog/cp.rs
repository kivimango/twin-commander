use super::{TransferProgress, TransferStrategy};
use fs_extra::dir::{
    copy_with_progress as copy_dir_with_progress, CopyOptions as DirCopyOptions,
    TransitProcess as DirTransitProcess,
};
use std::{
    path::{Path, PathBuf},
    sync::mpsc::Sender,
    thread,
    time::Duration,
};

pub struct CopyStrategy;

impl TransferStrategy for CopyStrategy {
    fn transfer_dir<P: AsRef<Path>>(
        &mut self,
        source: P,
        destination: P,
        tx: Sender<super::TransferProgress>,
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
            let _result = copy_dir_with_progress(
                AsRef::<Path>::as_ref(&from),
                AsRef::<Path>::as_ref(&to),
                &options,
                progress_handler,
            );
        });
    }

    fn transfer_file<P: AsRef<Path>>(
        &mut self,
        source: P,
        destination: P,
        tx: Sender<super::TransferProgress>,
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
            let _result = fs_extra::file::copy_with_progress(
                AsRef::<Path>::as_ref(&from),
                AsRef::<Path>::as_ref(&to),
                &options,
                progress_handler,
            );
        });
    }
}
