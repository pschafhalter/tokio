use std::time::{Duration, SystemTime};

use tokio;
use tokio::runtime;

async fn print_priority(priority: usize, sleep: Duration) {
    tokio::time::sleep(sleep).await;
    println!("{}", priority)
}

async fn async_main() {
    let sleep = Duration::from_millis(1000);
    tokio::spawn_with_spec(
        print_priority(100, sleep),
        tokio::TaskSpec {
            priority: 0,
            deadline: Some(SystemTime::now() + Duration::from_millis(100)),
        },
    );
    tokio::spawn_with_spec(
        print_priority(50, sleep),
        tokio::TaskSpec {
            priority: 0,
            deadline: Some(SystemTime::now() + Duration::from_millis(50)),
        },
    );

    tokio::time::sleep(Duration::from_millis(10000)).await;
}

fn main() {
    // let rt = runtime::Builder::new_current_thread()
    //     .enable_time()
    //     .build()
    //     .unwrap();
    let rt = runtime::Builder::new_multi_thread()
        .enable_time()
        .build()
        .unwrap();

    rt.block_on(async_main());
}
