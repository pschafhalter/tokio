use std::time::{Duration, Instant, SystemTime};

use rand::{self, Rng};
use tokio;
use tokio::runtime;

use futures::future::join_all;

async fn noop() {
    // println!("noop");
}

async fn time_tasks(num_tasks: usize) -> Duration {
    let mut rng = rand::thread_rng();
    let mut handles = Vec::with_capacity(num_tasks);

    let start = Instant::now();
    for _ in 0..num_tasks {
        let deadline_secs: u8 = rng.gen();
        let deadline = SystemTime::now() + Duration::from_secs(deadline_secs as u64);
        handles.push(tokio::spawn_with_deadline(noop(), deadline));
        // handles.push(tokio::spawn(noop()));
    }

    join_all(handles).await;
    // for (i, handle) in handles.into_iter().enumerate() {
    //     handle.await.unwrap();
    //     println!("{}", i);
    // }

    return start.elapsed();
}

async fn async_main() {
    // Warmup
    time_tasks(100).await;
    // Experiment
    let num_tasks = 1000;
    let duration = time_tasks(num_tasks).await;

    let throughput = (num_tasks as f64) / duration.as_secs_f64();
    println!("throughput: {} tasks / sec", throughput);
}

fn main() {
    let rt = runtime::Builder::new_multi_thread()
        .enable_time()
        .build()
        .unwrap();

    rt.block_on(async_main());
}
