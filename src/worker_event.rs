use crate::core::{
    list_dir::DirContent,
    worker::{ResultListener, SharedTask, TaskError, TaskResult, Worker},
};
use tuirealm::{
    listener::{ListenerResult, Poll},
    Event,
};

#[derive(PartialEq, Clone, PartialOrd)]
pub enum WorkerEvent {
    ListDirectoryResult(Result<Vec<DirContent>, TaskError>),
}

impl Eq for WorkerEvent {}

pub struct WorkerPort {
    worker: Worker,
    result_listener: ResultListener,
}

impl Poll<WorkerEvent> for WorkerPort {
    fn poll(&mut self) -> ListenerResult<Option<Event<WorkerEvent>>> {
        let mut task_result_lock = self.result_listener.lock().unwrap();
        let event = &mut *task_result_lock;
        let ev = match event {
            TaskResult::DirectoryListed(list_result) => Ok(Some(Event::User(
                WorkerEvent::ListDirectoryResult(list_result.clone()),
            ))),
            _ => Ok(None),
        };
        *event = TaskResult::Idle;
        return ev;
    }
}

impl WorkerPort {
    pub fn new(task: SharedTask) -> Self {
        let (worker, result_listener) = Worker::new(task);

        WorkerPort {
            worker,
            result_listener,
        }
    }
}
