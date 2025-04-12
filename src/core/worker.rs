use super::list_dir::{list_dir, DirContent, FilterOptions};
use std::{
    path::PathBuf,
    sync::{Arc, Condvar, Mutex},
    thread::JoinHandle,
};
use thiserror::Error;

pub type SharedTask = Arc<(Mutex<Task>, Condvar)>;
pub type ResultListener = Arc<Mutex<TaskResult>>;

pub enum Task {
    ListDirectory(PathBuf, FilterOptions),
    TransferFiles(FileTransferParams),
    DeleteFiles(Vec<PathBuf>),
    Shutdown,
    None,
}

pub enum TaskResult {
    DirectoryListed(Result<Vec<DirContent>, TaskError>),
    TransferProgressReport {
        files_transferred: usize,
        bytes_transferred: u64,
    },
    DeleteFilesResult(usize),
    Error(TaskError),
    Idle,
}

pub struct FileTransferParams {
    pub source: PathBuf,
    pub destination: PathBuf,
    pub keep_source: bool,
}

pub struct ProgressReport {
    bytes_transferred: u128,
    files_transferred: usize,
}

#[derive(Clone, Debug, Error, PartialEq, PartialOrd)]
pub enum TaskError {
    #[error("I/O error during processing task")]
    IoError,
}

impl From<std::io::Error> for TaskError {
    fn from(_value: std::io::Error) -> Self {
        TaskError::IoError
    }
}

pub struct TaskSender {
    task: SharedTask,
}

impl TaskSender {
    pub fn new(task: SharedTask) -> Self {
        TaskSender { task }
    }

    pub fn assign_task(&self, task: Task) {
        let (lock, cvar) = &*self.task;
        {
            let mut current_task = lock.lock().unwrap();
            *current_task = task;
        }
        cvar.notify_one();
    }
}
/// A background worker processing I/O related tasks off-loaded to another thread.
pub struct Worker {
    task: Arc<(Mutex<Task>, Condvar)>,
    result: ResultListener,
    worker: JoinHandle<()>,
}

impl Worker {
    /// Creates a new worker instance from a sender and a receiver.
    /// Sender will be send back the result of the processing of a task received from the receiver.
    /// Receiver will be listen for incoming tasks to process them and send back the result.
    pub fn new(task: SharedTask) -> (Self, ResultListener) {
        let task_clone = task.clone();
        let result = Arc::new(Mutex::new(TaskResult::Idle));
        let result_clone = Arc::clone(&result);
        let result_return = Arc::clone(&result);

        let worker = Worker {
            task,
            result,
            worker: std::thread::spawn(move || loop {
                let (task_lock, cvar) = &*task_clone;

                let mut current_task = cvar
                    .wait_while(task_lock.lock().unwrap(), |task| {
                        matches!(*task, Task::None)
                    })
                    .unwrap();

                let task_result = match &*current_task {
                    Task::ListDirectory(path, filter_options) => {
                        match list_dir(&path, &filter_options) {
                            Ok(list) => TaskResult::DirectoryListed(Result::Ok(list)),
                            Err(_error) => {
                                TaskResult::DirectoryListed(Result::Err(TaskError::IoError))
                            }
                        }
                    }
                    // Shouldn't happen due to `wait_while`
                    Task::None => TaskResult::Idle,
                    _ => TaskResult::Idle,
                };

                *result_clone.lock().unwrap() = task_result;
                *current_task = Task::None;
            }),
        };

        (worker, result_return)
    }
}
