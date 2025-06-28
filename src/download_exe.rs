use tokio::fs;

use crate::{AppState, FlashClientArg, RunninState};

pub async fn download_exe(state: &AppState, args: &FlashClientArg, files: &Vec<String>) {
    if files.is_empty() {
        return;
    }

    let running_state = &mut *state.running_state.lock().await;

    *running_state = RunninState::Downloading;

    let flash_addr = std::env::var("FLASH_ADDR").unwrap_or_else(|_| "localhost:4090".to_string());

    for file in files {
        let res = reqwest::get(format!("http://{}/executable/{}", flash_addr, file))
            .await
            .unwrap();

        let bytes = res.bytes().await.unwrap();

        fs::write(format!("{}/{}", args.out, file), bytes)
            .await
            .unwrap();
    }

    *running_state = RunninState::Downloaded;
}
