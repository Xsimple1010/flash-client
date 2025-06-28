use clap::Parser;
use serde::Deserialize;
use std::{sync::Arc, thread::JoinHandle, time::Duration};
use tokio::{sync::Mutex, time::sleep};

use crate::{
    download_exe::download_exe, execution_thread::ExecutionThread, update_exes::update_exes,
};

mod download_exe;
mod execution_thread;
mod update_exes;

#[derive(Debug, Default, Clone)]
struct AppState {
    current_exe: Arc<Mutex<Option<Executable>>>,
    dependencies: Arc<Mutex<Vec<Executable>>>,
    running_state: Arc<Mutex<RunninState>>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum RunninState {
    #[default]
    Downloading,
    Downloaded,
    HasPermission,
    Running,
}

#[derive(Debug, Default, Deserialize)]
struct Executable {
    pub name: String,
    pub hash: u64,
}

#[derive(Parser, Debug, Clone)]
struct FlashClientArg {
    #[arg(short, long)]
    executables: String,

    #[arg(short, long)]
    dependencies: Option<Vec<String>>,

    #[arg(short, long)]
    out: String,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let state = AppState::default();
    let args = FlashClientArg::parse();

    let mut _join: Option<JoinHandle<()>> = None;

    let mut exe_thread = ExecutionThread::new(state.running_state.clone(), args.clone());
    let flash_addr = std::env::var("FLASH_ADDR").unwrap_or_else(|_| "localhost:4090".to_string());

    println!("server listener on: {}", flash_addr);

    loop {
        let exes_to_update = update_exes(&state, &args).await;

        if !exes_to_update.is_empty() {
            exe_thread.stop().await;
        }

        download_exe(&state, &args, &exes_to_update).await;

        exe_thread.start().await;

        sleep(Duration::from_secs(2)).await;
    }
}
