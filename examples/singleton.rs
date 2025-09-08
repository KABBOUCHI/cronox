use cronox::facade::Schedule;

#[tokio::main]
async fn main() {
    Schedule::call(async || {
        println!("Every second! Time: {}", chrono::Utc::now());
    })
    .every_second();

    Schedule::call(async || {
        println!("Every five seconds! Time: {}", chrono::Utc::now());
    })
    .every_seconds(5);

    Schedule::call(async || {
        println!("Every minute! Time: {}", chrono::Utc::now());
    });

    Schedule::call(async || println!("Should be Skipped!"))
        .every_ten_seconds()
        .skip(true);

    Schedule::call(async || {
        println!("Not Skipped");
    })
    .every_ten_seconds()
    .skip(async || false);

    Schedule::run().await;
}
