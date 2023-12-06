use anyhow::{Context, Ok, Result};
use clap::{Parser, Subcommand};
use serde::Deserialize;
use serde_json::json;
use std::{collections::HashMap, error, fmt::format, io::Write};

#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u32,
}

pub async fn fetch_sl_token_with_refresh() -> Result<TokenResponse> {
    let client_id = std::env::var("CLIENT_ID").context("No CLIENT_ID found in environment")?;
    let refresh_token =
        std::env::var("REFRESH_TOKEN").context("no REFRESH_TOKEN found in environment")?;
    println!("Using client_id={}", client_id);
    println!("Using refresh_token={}", refresh_token);

    let mut data = HashMap::new();
    data.insert("grant_type", "refresh_token");
    data.insert("client_id", &client_id);
    data.insert("refresh_token", &refresh_token);

    let response = reqwest::Client::new()
        .post("https://api.dropbox.com/oauth2/token")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&data)
        .send()
        .await
        .context("Error requesting API auth token")?;

    Ok(response.json::<TokenResponse>().await?)
}

pub async fn fetch_api_zip<T: std::convert::AsRef<std::path::Path>>(
    token: &TokenResponse,
    source: &str,
    dest: T,
) -> Result<()> {
    let mut arg = HashMap::new();
    arg.insert("path", source);
    let json_val: serde_json::Value = serde_json::to_value(&arg)?;
    let json_str = serde_json::to_string(&json_val)?;

    println!("Using {} as api-arg", json_str);

    let client = reqwest::Client::builder()
        //.proxy(reqwest::Proxy::https("http://127.0.0.1:8080")?)
        .danger_accept_invalid_certs(true)
        .build()?;
    let resp = client
        .post("https://content.dropboxapi.com/2/files/download_zip")
        .header("Content-Type", "text/plain; charset=utf-8")
        .header("Authorization", format!("Bearer {}", token.access_token))
        .header("Dropbox-Api-Arg", &json_str)
        .send()
        .await
        .context("Error fetching ZIP file")?;

    let mut file = std::fs::File::create(dest)?;

    println!("{:?}", &resp);

    let mut bytes = resp.bytes().await?;
    file.write_all(&mut bytes)?;

    Ok(())
}

#[cfg(test)]
mod test {
    #[tokio::test]
    async fn dropbox_conn() {
        let _ = reqwest::Client::new()
            .post("https://api.dropbox.com/oauth2/token")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .send()
            .await
            .unwrap();
    }
}
