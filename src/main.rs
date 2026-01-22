use anyhow::{Result, anyhow};
use chrono::prelude::*;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::{fs::File, io::Read};

use axum::{
    Json,
    extract::{Extension, Query, rejection::JsonRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{Router, get, post},
};
use serde_json::{Value, json};

mod handler;
use handler::*;

use crate::handler::get_mah_token::MahTokenResp;

mod schedule;
// use schedule::*;

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
    pub post_server_url: String,
    pub paper_server_url: String,
    pub tv_server_url: String,
    pub docstatus: u32,
    pub site_id: u32,
    pub get_paper_articles_interval: u64,
    pub tv_columnid: String,
    pub daemon: bool,
    pub timestamp_extra: u32,
    pub mah_token_server_url: String,
    pub mah_search_server_url: String,
    pub mah_content_server_url: String,
    pub mah_client_id: String,
    pub mah_client_secret: String,
    pub grant_type: String,
    pub loop_token_interval: u64,
    pub mah_username: String,
    pub search_char_at_least: usize,
    pub loop_tv_url_interval: u64,
    pub loop_tv_url_year: Option<u32>,
    pub loop_tv_url_month: Option<u32>,
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
    let sobey_token = Arc::new(Mutex::new(MahTokenResp::default()));
    let token_daemon = tokio::task::spawn(handler::get_mah_token::loop_get_access_token(
        Arc::clone(&cfg),
        Arc::clone(&sobey_token),
    ));
    let loop_tv_url_daemon = tokio::task::spawn(handler::search_material::loop_get_tv_url(
        cfg.loop_tv_url_year,
        cfg.loop_tv_url_month,
        cfg.mah_search_server_url.clone(),
        cfg.mah_content_server_url.clone(),
        cfg.post_server_url.clone(),
        cfg.loop_tv_url_interval,
        Arc::clone(&sobey_token),
    ));

    // let paper_daemon = if cfg.daemon {
    //     Some(tokio::task::spawn(
    //         get_paper_articles_task::get_paper_articles_task(Arc::clone(&cfg)),
    //     ))
    // } else {
    //     None
    // };

    let app = Router::new()
        .route("/", get(async || "hello, ascm article!".to_string()))
        // .route("/get_paper_articles", get(get_paper_articles::GetArticles::handle_get))
        // .route("/get_paper_article_detail", get(get_paper_article_detail::GetArticleDetail::handle_get))
        .route(
            "/get_tv_newslists",
            get(get_tv_newslists::GetTvNewsLists::handle_get),
        )
        .route(
            "/get_tv_newslist_detail",
            get(get_tv_newslist_detail::GetTvNewsListDetail::handle_get),
        )
        // .route(
        //     "/get_mah_token",
        //     get(get_mah_token::GetMahToken::handle_get),
        // )
        // .route("/get_tv_article", get(get_tv_article::GetTvArticle::handle_get))
        .route(
            "/postPaperArticles",
            get(post_paper_articles::PostPaperArticles::handle_get),
        )
        .route(
            "/postTvArticles",
            get(post_tv_articles::PostTvArticles::handle_get),
        )
        .route(
            "/getTvReporters",
            get(get_tv_reporters::GetTvReporters::handle_get),
        )
        .route(
            "/searchMahNews",
            post(search_material::SearchMaterail::handle_post_with_token),
        )
        .layer(Extension(Arc::clone(&cfg)))
        .layer(Extension(Arc::clone(&sobey_token)));
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", cfg.port)).await?;
    axum::serve(listener, app).await?;

    // if let Some(pd) = paper_daemon {
    //     let _ = pd.await?;
    // }

    let _ = token_daemon.await?;
    let _ = loop_tv_url_daemon.await?;

    Ok(())
}
