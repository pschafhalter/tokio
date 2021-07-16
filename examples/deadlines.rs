use std::time::{Duration, SystemTime};

use tokio;
use tokio::runtime;
use tokio::time;

async fn print_priority(priority: usize) {
    println!("{}", priority)
}

async fn async_main() {
    tokio::spawn_with_deadline(
        print_priority(100),
        SystemTime::now() + Duration::from_millis(100),
    );
    tokio::spawn_with_deadline(
        print_priority(50),
        SystemTime::now() + Duration::from_millis(50),
    );

    tokio::time::sleep(Duration::from_millis(100)).await;
}

fn main() {
    let basic_rt = runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .unwrap();

    basic_rt.block_on(async_main());
}
