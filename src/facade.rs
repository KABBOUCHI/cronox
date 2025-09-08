use std::{
    future::Future,
    sync::{OnceLock, RwLock},
};

use super::{ScheduleIndex, Scheduler};

static SCHEDULER: OnceLock<RwLock<Scheduler>> = OnceLock::new();

pub struct Schedule;

impl Schedule {
    pub fn get_instance() -> &'static RwLock<Scheduler> {
        SCHEDULER.get_or_init(|| RwLock::new(Scheduler::new()))
    }

    pub fn call<F, Fut>(callback: F) -> Scheduler<ScheduleIndex>
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + Sync + 'static,
    {
        let mut scheduler = Self::get_instance()
            .write()
            .expect("Failed to acquire write lock");

        let res = scheduler.call(callback);

        res
    }

    pub fn command(
        name: impl Into<String>,
        args: Vec<impl Into<String>>,
    ) -> Scheduler<ScheduleIndex> {
        let mut scheduler = Self::get_instance()
            .write()
            .expect("Failed to acquire write lock");

        let args: Vec<String> = args.into_iter().map(|arg| arg.into()).collect();

        let res = scheduler.command(name.into(), args);

        res
    }

    pub async fn run() {
        let mut scheduler = Self::get_instance()
            .write()
            .expect("Failed to acquire write lock");

        scheduler.run().await;
    }
}
