use clap::Parser;
use serde::Deserialize;
use serde_json::from_str;
use std::{sync::Arc, time::Duration};
use tokio::{sync::Mutex, time::sleep};

#[derive(Debug, Default, Clone)]
struct AppState {
    quee: Arc<Mutex<Executable>>,
}

#[derive(Debug, Default, Deserialize)]
struct Executable {
    pub name: String,
    pub hash: u64,
}

#[derive(Parser, Debug)]
struct FlashClientArg {
    #[arg(short, long)]
    executables: Vec<String>,

    #[arg(short, long)]
    out: String,
}

#[tokio::main]
async fn main() {
    let state = AppState::default();
    let args = FlashClientArg::parse();

    println!("{:?}", args.executables);

    loop {
        get_build_status(&state).await;
        sleep(Duration::from_secs(2)).await;
    }
}

async fn get_build_status(state: &AppState) {
    let res = reqwest::get("http://localhost:4090/executables")
        .await
        .unwrap();

    let data = match res.text().await {
        Ok(data) => data,
        Err(err) => {
            eprintln!("Error fetching data: {}", err);
            return;
        }
    };

    let executables = match from_str::<Vec<Executable>>(&data) {
        Ok(exe) => exe,
        Err(err) => {
            eprintln!("Error parsing JSON: {}", err);
            return;
        }
    };

    println!("{:?}", executables)
}

async fn run_exe() {}

async fn append_run_quee(state: AppState) {}
