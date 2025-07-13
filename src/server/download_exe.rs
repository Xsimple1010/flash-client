use crate::AppState;

use axum::{
    extract::{Multipart, State},
    http::StatusCode,
};
use tokio::{fs, io::AsyncWriteExt};

// Função que lida com o upload do arquivo
pub async fn download_exe(
    State(state): State<AppState>,
    multipart: Multipart,
) -> Result<(), (StatusCode, String)> {
    let mut thread = state.execution_thread.lock().await;
    thread.stop().await;

    let file_name = download(&state, multipart).await?;

    thread.start(file_name).await;

    Ok(())
}

// Função que lida com o upload do arquivo
pub async fn download_dep(
    State(state): State<AppState>,
    multipart: Multipart,
) -> Result<(), (StatusCode, String)> {
    let mut thread = state.execution_thread.lock().await;
    thread.stop().await;

    let _ = download(&state, multipart).await?;

    Ok(())
}

async fn download(
    state: &AppState,
    mut multipart: Multipart,
) -> Result<String, (StatusCode, String)> {
    // Itera sobre as partes do formulário multipart
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Erro ao processar o formulário: {}", e),
        )
    })? {
        // Nome do campo no formulário
        let _name = field.name().map(|n| n.to_string()).unwrap_or_default();
        // Nome do arquivo (se fornecido)
        let file_name = field.file_name().map(|n| n.to_string()).unwrap_or_default();
        // Dados do arquivo
        let data = field.bytes().await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Erro ao ler os dados do arquivo: {}", e),
            )
        })?;

        // Define o caminho onde o arquivo será salvo
        let save_path = format!("{}/{}", state.out.clone(), file_name);

        // Cria o diretório "uploads" se não existir
        tokio::fs::create_dir_all(state.out.clone())
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Erro ao criar diretório: {}", e),
                )
            })?;

        fs::remove_file(save_path.clone()).await.ok(); // Ignora erros ao remover o arquivo, se não existir

        // Salva o arquivo no sistema de arquivos
        let mut file = fs::File::create(&save_path).await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Erro ao criar o arquivo: {}", e),
            )
        })?;
        file.write_all(&data).await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Erro ao salvar o arquivo: {}", e),
            )
        })?;

        println!(
            "Arquivo '{}' salvo com sucesso em '{}'",
            file_name, save_path
        );
        return Ok(file_name);
    }

    Err((
        StatusCode::BAD_REQUEST,
        "Nenhum arquivo encontrado no formulário".to_string(),
    ))
}
