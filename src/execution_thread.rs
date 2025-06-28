use std::{
    process::Command,
    sync::{Arc, RwLock},
    thread::{self, sleep},
    time::Duration,
};

use tokio::sync::Mutex;

use crate::{FlashClientArg, RunninState};

pub struct ExecutionThread {
    thread_is_running: Arc<RwLock<bool>>,
    running_state: Arc<Mutex<RunninState>>,
    args: FlashClientArg,
}

impl Drop for ExecutionThread {
    fn drop(&mut self) {
        match self.thread_is_running.write() {
            Ok(mut running) => {
                *running = false;
            }
            Err(op) => {
                let mut running = op.into_inner();
                *running = false;
            }
        }
    }
}

impl ExecutionThread {
    pub fn new(running_state: Arc<Mutex<RunninState>>, args: FlashClientArg) -> Self {
        Self {
            thread_is_running: Arc::new(RwLock::new(false)),
            running_state,
            args,
        }
    }

    fn set_permission(&self) -> bool {
        let output = Command::new("chmod")
            .arg("+x")
            .arg(format!("{}/{}", self.args.out, self.args.executables))
            .output()
            .expect("Não foi possível executar o programa");

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("Erro no build: {}", stderr);
            return false;
        }

        true
    }

    pub async fn start(&mut self) {
        let mut state = {
            let d = &*self.running_state.lock().await;
            d.clone()
        };

        if state == RunninState::Running {
            return;
        }

        // permissão para executar o programa
        if state == RunninState::Downloaded {
            if self.set_permission() {
                {
                    let n_state = &mut *self.running_state.lock().await;
                    *n_state = RunninState::HasPermission;

                    state = n_state.clone();
                }
            }
        }

        if state == RunninState::HasPermission {
            {
                let mut thread_is_running = self.thread_is_running.write().expect("msg");
                *thread_is_running = true;
            }

            self.thread_handle().await;
        }
    }

    pub async fn stop(&self) {
        let mut thread_is_running = self.thread_is_running.write().expect("msg");
        *thread_is_running = false;

        sleep(Duration::from_secs(3));
    }

    async fn thread_handle(&mut self) {
        let thread_is_running = self.thread_is_running.clone();
        let executables = self.args.executables.clone();
        let out = self.args.out.clone();

        thread::spawn(move || {
            let mut output = Command::new(format!("{}/{}", out, executables))
                .spawn()
                .expect("Não foi possível executar o programa");

            loop {
                match thread_is_running.read() {
                    Ok(is_running) => {
                        if !*is_running {
                            break;
                        }
                    }
                    Err(op) => {
                        let mut _is_running = *op.into_inner();
                        _is_running = false;
                        break;
                    }
                };

                sleep(Duration::from_secs(1));
            }

            output.kill().expect("Não foi possível matar o processo");
        });

        *self.running_state.lock().await = RunninState::Running;
    }
}
