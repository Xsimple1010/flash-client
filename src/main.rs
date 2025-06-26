use clap::Parser;
use serde::Deserialize;
use serde_json::from_str;
use std::{sync::Arc, time::Duration};
use tokio::{sync::Mutex, time::sleep};

#[derive(Debug, Default, Clone)]
struct AppState {
    current_exe: Arc<Mutex<Option<Executable>>>,
    depencies: Arc<Mutex<Vec<Executable>>>,
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
    dependecies: Option<Vec<String>>,

    #[arg(short, long)]
    out: String,
}

#[tokio::main]
async fn main() {
    let state = AppState::default();
    let args = FlashClientArg::parse();

    println!("{:?}", args.executables);

    loop {
        get_build_status(&state, &args).await;
        sleep(Duration::from_secs(2)).await;
    }
}

async fn get_build_status(state: &AppState, args: &FlashClientArg) {
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

    let avalible_exes = match from_str::<Vec<Executable>>(&data) {
        Ok(exe) => exe,
        Err(err) => {
            eprintln!("Error parsing JSON: {}", err);
            return;
        }
    };

    for exe in avalible_exes {
        if args.executables.eq(&exe.name) {
            // executavel princiapal encontrado!
            let current_exe = &mut *state.current_exe.lock().await;

            match current_exe {
                Some(current_exe) => {
                    if current_exe.hash != exe.hash {
                        current_exe.hash = exe.hash;
                        current_exe.name = exe.name;
                    }
                }
                None => {
                    current_exe.replace(Executable {
                        name: exe.name.clone(),
                        hash: exe.hash,
                    });
                }
            }

            continue;
        }

        if let Some(depencies) = &args.dependecies {
            if depencies.contains(&exe.name) {
                let current_depencies = state.depencies.lock().await;
            }
        }
    }

    // println!("{:?}", avalible_exes)
}

async fn run_exe() {}

async fn append_run_quee(state: AppState) {}
