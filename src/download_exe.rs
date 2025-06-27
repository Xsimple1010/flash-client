use tokio::fs;

use crate::{AppState, DownloadState, FlashClientArg};

pub async fn download_exe(state: &AppState, args: &FlashClientArg, files: &Vec<String>) {
    let download_state = &mut *state.download_state.lock().await;

    *download_state = DownloadState::Progress;

    for file in files {
        let res = reqwest::get(format!("http://localhost:4090/executable/{}", file))
            .await
            .unwrap();

        let bytes = res.bytes().await.unwrap();

        fs::write(format!("{}/{}", args.out, file), bytes)
            .await
            .unwrap();
    }

    *download_state = DownloadState::Completed;
}
