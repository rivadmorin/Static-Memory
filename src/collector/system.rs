use tokio::sync::mpsc;
use tokio::time::{self, Duration};
use crate::models::{InputEvent, SystemMetrics};
use sysinfo::System;
use chrono::Utc;

pub async fn run_system_collector(tx: mpsc::Sender<InputEvent>) {
    let mut sys = System::new_all();
    let mut interval = time::interval(Duration::from_secs(2));

    loop {
        interval.tick().await;

        sys.refresh_cpu();
        sys.refresh_memory();

        let metrics = SystemMetrics {
            cpu_usage: sys.global_cpu_info().cpu_usage(),
            memory_usage_mb: sys.used_memory() / 1024 / 1024,
            timestamp: Utc::now(),
        };

        let _ = tx.send(InputEvent::System(metrics)).await;
    }
}
