use clap::Parser;
use std::{sync::Arc, thread::JoinHandle};
use tokio::sync::Mutex;

use crate::{execution_thread::ExecutionThread, server::init_server};

mod execution_thread;
mod server;

#[derive(Clone)]
struct AppState {
    execution_thread: Arc<Mutex<ExecutionThread>>,
    out: String,
}

impl AppState {
    fn new(args: FlashClientArg) -> Self {
        Self {
            out: args.out.clone(),
            execution_thread: Arc::new(Mutex::new(ExecutionThread::new(args))),
        }
    }
}

#[derive(Parser, Debug, Clone)]
struct FlashClientArg {
    #[arg(short, long)]
    out: String,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let args = FlashClientArg::parse();
    let state = AppState::new(args);

    let mut _join: Option<JoinHandle<()>> = None;

    init_server(state).await
}
