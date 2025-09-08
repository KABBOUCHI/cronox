use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
};

use chrono::{TimeZone, Utc};

pub mod facade;

type CallbackFn = Arc<dyn Fn() -> Pin<Box<dyn Future<Output = ()> + Send + Sync>> + Send + Sync>;

#[derive(Clone)]
pub enum ScheduleType {
    ScheduleCommand(String, Vec<String>),
    ScheduleCallback(CallbackFn),
}

#[derive(Clone)]
pub struct Schedule {
    cron: Arc<Mutex<cron::Schedule>>,
    typ: ScheduleType,
}

impl Schedule {
    async fn run(&mut self) {
        match &self.typ {
            ScheduleType::ScheduleCommand(name, args) => {
                let mut cmd = tokio::process::Command::new(name);
                cmd.args(args);
                cmd.stdout(std::process::Stdio::null());
                cmd.stderr(std::process::Stdio::null());
                let _ = cmd.spawn();
            }
            ScheduleType::ScheduleCallback(callback) => {
                (callback)().await;
            }
        }
    }
}

impl Schedule {
    pub fn from_callback<F, Fut>(callback: F) -> Schedule
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + Sync + 'static,
    {
        let callback_arc = Arc::new(move || -> Pin<Box<dyn Future<Output = ()> + Send + Sync>> {
            Box::pin(callback())
        });

        Schedule {
            cron: Arc::new(Mutex::new("0 * * * * *".parse().unwrap())),
            typ: ScheduleType::ScheduleCallback(callback_arc),
        }
    }

    pub fn from_command(name: String, args: Vec<String>) -> Schedule {
        Schedule {
            cron: Arc::new(Mutex::new("0 * * * * *".parse().unwrap())),
            typ: ScheduleType::ScheduleCommand(name, args),
        }
    }
}

pub struct DefaultContext;

pub struct Scheduler<T = DefaultContext, Tz = Utc>
where
    Tz: TimeZone,
{
    pub(crate) schedules: Vec<Schedule>,
    context: T,
    timezone: Tz,
}

impl Scheduler {
    pub fn new() -> Scheduler {
        Scheduler {
            schedules: Vec::new(),
            context: DefaultContext,
            timezone: Utc,
        }
    }
}

impl<T> Scheduler<T> {
    pub fn context(&self) -> &T {
        &self.context
    }

    pub fn schedules(&mut self) -> &mut Vec<Schedule> {
        &mut self.schedules
    }

    pub fn reborrow(&mut self) -> Scheduler {
        Scheduler {
            schedules: self.schedules.clone(),
            context: DefaultContext,
            timezone: self.timezone.clone(),
        }
    }
}

impl Scheduler<DefaultContext> {
    pub fn call<F, Fut>(&mut self, callback: F) -> Scheduler<ScheduleIndex>
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + Sync + 'static,
    {
        let schedule = Schedule::from_callback(callback);
        self.schedules.push(schedule);

        Scheduler {
            schedules: self.schedules.clone(),
            context: ScheduleIndex(self.schedules.len() - 1),
            timezone: self.timezone.clone(),
        }
    }

    pub fn command(
        &mut self,
        name: impl Into<String>,
        args: Vec<String>,
    ) -> Scheduler<ScheduleIndex> {
        let schedule = Schedule::from_command(name.into(), args);
        self.schedules.push(schedule);

        Scheduler {
            schedules: self.schedules.clone(),
            context: ScheduleIndex(self.schedules.len() - 1),
            timezone: self.timezone.clone(),
        }
    }
    pub async fn run(&mut self) {
        loop {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {
                    break;
                }
                _ = tokio::time::sleep(tokio::time::Duration::from_secs(1)) => {
                    let now = Utc::now();

                    let mut tasks = Vec::new();

                    for schedule in &mut self.schedules {
                        let cron = schedule.cron.lock().unwrap();
                        if let Some(next) = cron.upcoming(Utc).next() {
                            let diff = next.signed_duration_since(now);
                            if diff.num_seconds() <= 0 && diff.num_seconds() > -1 {
                                drop(cron);
                                let mut schedule_clone = schedule.clone();
                                tasks.push(tokio::spawn(async move {
                                    schedule_clone.run().await;
                                }));
                            }
                        }
                    }

                    if !tasks.is_empty() {
                        let _ = futures::future::join_all(tasks).await;
                    }
                }
            }
        }
    }
}

pub struct ScheduleIndex(usize);

impl Scheduler<ScheduleIndex> {
    fn splice_into_position(&mut self, position: usize, value: &str) -> &mut Self {
        let expression = self.expression();
        let segments: Vec<&str> = expression.split(' ').collect();

        let mut segments: Vec<String> = segments.into_iter().map(|s| s.to_string()).collect();
        segments[position] = value.to_string();

        let new_expression = segments.join(" ");

        self.cron(&new_expression)
    }

    fn index(&self) -> usize {
        self.context().0
    }

    fn expression(&self) -> String {
        let index = self.index();
        let cron = self.schedules[index].cron.lock().unwrap();

        cron.to_string()
    }

    pub fn cron(&mut self, expression: &str) -> &mut Self {
        let index = self.index();

        *self.schedules[index].cron.lock().unwrap() = expression.parse().unwrap();

        self
    }

    pub fn every_second(&mut self) -> &mut Self {
        self.splice_into_position(0, "*")
    }

    pub fn every_seconds(&mut self, second: u8) -> &mut Self {
        self.splice_into_position(0, &format!("*/{}", second))
    }

    pub fn every_five_seconds(&mut self) -> &mut Self {
        self.every_seconds(5)
    }

    pub fn every_ten_seconds(&mut self) -> &mut Self {
        self.every_seconds(10)
    }

    pub fn every_fifteen_seconds(&mut self) -> &mut Self {
        self.every_seconds(15)
    }

    pub fn every_thirty_seconds(&mut self) -> &mut Self {
        self.every_seconds(30)
    }

    pub fn every_minutes(&mut self, minutes: u8) -> &mut Self {
        self.splice_into_position(1, &format!("*/{}", minutes))
    }

    pub fn every_minute(&mut self) -> &mut Self {
        self.splice_into_position(1, "*")
    }

    pub fn every_two_minutes(&mut self) -> &mut Self {
        self.every_minutes(2)
    }

    pub fn every_three_minutes(&mut self) -> &mut Self {
        self.every_minutes(3)
    }

    pub fn every_four_minutes(&mut self) -> &mut Self {
        self.every_minutes(4)
    }

    pub fn every_five_minutes(&mut self) -> &mut Self {
        self.every_minutes(5)
    }

    pub fn every_ten_minutes(&mut self) -> &mut Self {
        self.every_minutes(10)
    }

    pub fn every_fifteen_minutes(&mut self) -> &mut Self {
        self.every_minutes(15)
    }

    pub fn every_thirty_minutes(&mut self) -> &mut Self {
        self.every_minutes(30)
    }
}
