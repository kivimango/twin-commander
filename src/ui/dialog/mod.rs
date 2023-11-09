use fs_extra::{
    dir::TransitProcess as DirTransitProcess, file::TransitProcess as FileTransitProcess,
};
use std::path::Path;
use std::sync::mpsc::Sender;

mod cp;
mod help;
mod menu;
mod mkdir;
mod mv;
mod rm;
mod transfer;

pub use self::cp::*;
pub use self::help::*;
pub use self::menu::*;
pub use self::mkdir::*;
pub use self::mv::*;
pub use self::rm::*;
pub use self::transfer::*;

/// Abstraction of file transfers (copy/move) for reusing
/// the same TransferDialog fo every different file transfers.
pub trait TransferStrategy {
    fn transfer_dir<P: AsRef<Path>>(
        &mut self,
        source: P,
        destination: P,
        tx: Sender<TransferProgress>,
    );
    fn transfer_file<P: AsRef<Path>>(
        &mut self,
        source: P,
        destination: P,
        tx: Sender<TransferProgress>,
    );
}

// Convenient type for sending two different type of data through a channel:
// dont need two distinct (tx,rx)
pub enum TransferProgress {
    DirTransfer(DirTransitProcess),
    FileTransfer(FileTransitProcess),
    None,
}
