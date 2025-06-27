use clap::Parser;
use serde::Deserialize;
use std::{process::Command, sync::Arc, time::Duration};
use tokio::{sync::Mutex, task, time::sleep};

use crate::{download_exe::download_exe, update_exes::update_exes};

mod download_exe;
mod update_exes;

#[derive(Debug, Default, Clone)]
struct AppState {
    current_exe: Arc<Mutex<Option<Executable>>>,
    dependencies: Arc<Mutex<Vec<Executable>>>,
    download_state: Arc<Mutex<DownloadState>>,
}

#[derive(Debug, Clone, Default)]
pub enum DownloadState {
    #[default]
    Progress,
    Completed,
}

#[derive(Debug, Default, Deserialize)]
struct Executable {
    pub name: String,
    pub hash: u64,
}

#[derive(Parser, Debug)]
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
    let state = AppState::default();
    let args = FlashClientArg::parse();

    loop {
        let exes_to_update = update_exes(&state, &args).await;
        download_exe(&state, &args, &exes_to_update).await;

        sleep(Duration::from_secs(2)).await;

        println!("update: {:?}", exes_to_update);
        println!("exe: {:?}", state.current_exe.lock().await);
        println!("dep: {:?}", state.dependencies.lock().await);
    }
}

async fn run_exe(state: AppState, dir: String, exe_name: String) {
    tokio::spawn(async move {
        loop {
            // if *state.download_state.lock().await != DownloadState::Completed {
            //     return;
            // }
        }

        let output = task::spawn_blocking(move || {
            Command::new("chmod")
                .arg("+x")
                .arg(format!("{}/{}", dir, exe_name))
                .output()
                .expect("Não foi possível executar o programa")
        })
        .await
        .expect("falha na task que lida com a execução");

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("Erro no build: {}", stderr);
            return;
        }
    });
}
