use clap::Parser;
use serde::Deserialize;
use std::{sync::Arc, time::Duration};
use tokio::{sync::Mutex, time::sleep};

#[derive(Debug, Default, Clone)]
struct AppState {
    quee: Arc<Mutex<Executable>>,
}

#[derive(Debug, Default, Deserialize)]
struct Executable {
    pub nema: String,
    pub hash: u64,
}

#[derive(Parser, Debug)]
struct FlashClientArg {
    #[arg(short, long)]
    executables: Vec<String>,
}

#[tokio::main]
async fn main() {
    let state = AppState::default();
    let args = FlashClientArg::parse();

    println!("{:?}", args.executables);

    let _ = tokio::spawn(async move {
        loop {
            get_build_status(&state).await;
            sleep(Duration::from_secs(2)).await;
        }
    })
    .await;
}

async fn get_build_status(state: &AppState) {
    let res = reqwest::get("http://localhost:4090/executables")
        .await
        .unwrap();

    println!("{:?}", res.text().await.unwrap())
}

async fn run_exe() {}

async fn append_run_quee(state: AppState) {}
