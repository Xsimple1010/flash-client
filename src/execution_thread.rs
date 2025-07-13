use std::{
    process::Command,
    sync::{Arc, RwLock},
    thread::{self, JoinHandle, sleep},
    time::Duration,
};

use crate::FlashClientArg;

pub struct ExecutionThread {
    pub thread_is_running: Arc<RwLock<bool>>,
    args: FlashClientArg,
    join_handle: Option<JoinHandle<()>>,
}

impl Drop for ExecutionThread {
    fn drop(&mut self) {
        // Definir thread_is_running como false
        match self.thread_is_running.write() {
            Ok(mut running) => *running = false,
            Err(op) => *op.into_inner() = false,
        }

        // Tentar juntar a thread, se ela existir
        if let Some(handle) = self.join_handle.take() {
            let mut retry_count = 0;
            const MAX_RETRIES: u32 = 5;

            while retry_count < MAX_RETRIES {
                if handle.is_finished() {
                    println!("Thread stopped successfully.");
                    return;
                }
                sleep(Duration::from_secs(1));
                retry_count += 1;
            }

            // Tentar juntar a thread
            match handle.join() {
                Ok(_) => println!("Thread stopped successfully after join."),
                Err(_) => eprintln!(
                    "Thread panicked or failed to stop after {} retries.",
                    MAX_RETRIES
                ),
            }
        } else {
            println!("No thread to stop.");
        }
    }
}

impl ExecutionThread {
    pub fn new(args: FlashClientArg) -> Self {
        Self {
            thread_is_running: Arc::new(RwLock::new(false)),
            args,
            join_handle: None,
        }
    }

    async fn set_permission(&self, exe_name: &String) -> bool {
        let output = Command::new("chmod")
            .arg("+x")
            .arg(format!("{}/{}", self.args.out, exe_name))
            .output()
            .expect("Não foi possível executar o programa");

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("Erro no build: {}", stderr);
            return false;
        }
        true
    }

    pub async fn start(&mut self, exe_name: String) {
        let state = {
            self.thread_is_running
                .read()
                .expect("Failed to acquire read lock")
                .clone()
        };

        if state {
            return;
        }

        // Definir permissões, se necessário
        if !self.set_permission(&exe_name).await {
            eprintln!("Failed to set permissions for {}", exe_name);
            return;
        }

        {
            *self
                .thread_is_running
                .write()
                .expect("Failed to acquire write lock") = true;
        }

        self.thread_handle(exe_name).await;
    }

    pub async fn stop(&mut self) {
        {
            *self
                .thread_is_running
                .write()
                .expect("Failed to acquire write lock") = false
        }

        sleep(Duration::from_secs(1));
        // Tentar juntar a thread, se ela existir
        if let Some(handle) = self.join_handle.take() {
            match handle.join() {
                Ok(_) => println!("Thread stopped successfully."),
                Err(_) => eprintln!("Thread panicked or failed to stop."),
            }
        }
    }

    async fn thread_handle(&mut self, exe_name: String) {
        let thread_is_running = self.thread_is_running.clone();
        let out = self.args.out.clone();

        let handle = thread::spawn(move || {
            println!(
                "Iniciando thread para executar o programa: {}",
                format!("{}/{}", out, exe_name)
            );
            let mut output = Command::new("./bin/obdir")
                .spawn()
                .expect("Não foi possível executar o programa");

            loop {
                if let Ok(is_running) = thread_is_running.read() {
                    if !*is_running {
                        break;
                    }
                } else {
                    break;
                }

                // Verificar se o processo terminou naturalmente
                if let Ok(Some(_)) = output.try_wait() {
                    break;
                }

                sleep(Duration::from_secs(1));
            }

            // Tentar encerrar o processo
            let _ = output.kill();
            let _ = output.wait(); // Garantir que o processo foi finalizado
        });

        // Armazenar o JoinHandle
        self.join_handle = Some(handle);
    }
}
