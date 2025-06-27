use serde_json::from_str;

use crate::{AppState, Executable, FlashClientArg};

async fn request_available_exes() -> Vec<Executable> {
    let res = reqwest::get("http://localhost:4090/executables")
        .await
        .unwrap();

    let exes = Vec::new();

    let data = match res.text().await {
        Ok(data) => data,
        Err(err) => {
            eprintln!("Error fetching data: {}", err);
            return exes;
        }
    };

    match from_str::<Vec<Executable>>(&data) {
        Ok(exe) => exe,
        Err(err) => {
            eprintln!("Error parsing JSON: {}", err);
            return exes;
        }
    }
}

pub async fn update_exes(state: &AppState, args: &FlashClientArg) -> Vec<String> {
    let available_exes = request_available_exes().await;

    let mut need_update = Vec::new();

    for exe in available_exes {
        if args.executables.eq(&exe.name) {
            // executable principal encontrado!
            let current_exe = &mut *state.current_exe.lock().await;

            match current_exe {
                Some(current_exe) => {
                    if current_exe.hash != exe.hash {
                        current_exe.hash = exe.hash;
                        current_exe.name = exe.name.clone();

                        need_update.push(exe.name);
                    }
                }
                None => {
                    current_exe.replace(Executable {
                        name: exe.name.clone(),
                        hash: exe.hash,
                    });

                    need_update.push(exe.name);
                }
            }

            continue;
        }

        let dependencies = match &args.dependencies {
            Some(dependencies) => dependencies,
            None => continue,
        };

        // verifica se o executável atual é uma dependência
        if dependencies.contains(&exe.name) {
            let current_dependencies = &mut *state.dependencies.lock().await;

            // se for a primeira dependência
            if current_dependencies.is_empty() {
                current_dependencies.push(Executable {
                    name: exe.name.clone(),
                    hash: exe.hash,
                });
                need_update.push(exe.name.clone());
                continue;
            }

            let mut need_add_new_dependencies = true;

            // verifica se é necessário atualizar a dependência
            for item in &mut *current_dependencies {
                if item.name == exe.name {
                    if item.hash != exe.hash {
                        item.hash = exe.hash;
                        need_update.push(exe.name.clone());
                    }

                    need_add_new_dependencies = false;
                    break;
                }
            }

            if need_add_new_dependencies {
                current_dependencies.push(Executable {
                    name: exe.name.clone(),
                    hash: exe.hash,
                });

                need_update.push(exe.name.clone());
            }
        }
    }

    need_update
}
