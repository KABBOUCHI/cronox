use cronox::Scheduler;

#[tokio::main]
async fn main() {
    let mut scheduler = Scheduler::new();

    scheduler
        .call(async || {
            println!("Every second! Time: {}", chrono::Utc::now());
        })
        .every_second();

    scheduler
        .call(async || {
            println!("Every five seconds! Time: {}", chrono::Utc::now());
        })
        .every_seconds(5);

    scheduler.call(async || {
        println!("Every minute! Time: {}", chrono::Utc::now());
    });

    scheduler
        .call(async || println!("Executing...."))
        .every_ten_minutes()
        .immediately();

    scheduler.run().await;
}
