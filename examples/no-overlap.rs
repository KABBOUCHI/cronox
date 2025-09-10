use cronox::Scheduler;

#[tokio::main]
async fn main() {
    let mut scheduler = Scheduler::new();

    scheduler
        .call(async || {
            println!("Every second! Time: {}", chrono::Utc::now());

            tokio::time::sleep(std::time::Duration::from_secs(10)).await;

            println!("Done!");
        })
        .every_second()
        .without_overlapping();

    scheduler.run().await;
}
