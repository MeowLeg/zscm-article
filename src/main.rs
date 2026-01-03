use anyhow::{Result, anyhow};
use axum::{
    Extension,
    routing::{Router, get},
};
use chrono::prelude::*;
use clap::Parser;
use serde::{Deserialize, Serialize};
use sqlx::{Connection, SqliteConnection};
use std::sync::Arc;
use std::{fs::File, io::Read};

mod handler;
use handler::*;

#[derive(Debug, Parser)]
#[command(version, about, long_about=None)]
struct Cli {
    /// config file
    #[arg(short, long, default_value = "./config.toml")]
    config: String,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub port: u32,
    pub db_path: String,
    pub server_url: String,
}

pub fn read_from_toml(f: &str) -> Result<Config> {
    let mut file = File::open(f)?;
    let mut s = String::new();
    file.read_to_string(&mut s)?;
    let config: Config = toml::from_str(&s)?;
    Ok(config)
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let cfg = Arc::new(read_from_toml(&cli.config)?);
    let app = Router::new()
        .route("/", get(async || "hello, ascm article!".to_string()))
        .layer(Extension(Arc::clone(&cfg)));
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", cfg.port)).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
