use super::*;
use chrono::Duration as ChDuration;
use regex::Regex;
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::{Duration, sleep};
// use sqlx::{Connection, FromRow, Sqlite, SqliteConnection};

pub mod get_paper_article_detail;
pub mod get_paper_articles;
pub mod get_tv_article;
pub mod get_tv_newslist_detail;
pub mod get_tv_newslists;
pub mod get_tv_reporters;

pub mod post_paper_articles;
pub mod post_tv_articles;

pub mod get_mah_token;
pub mod search_material;

use get_paper_article_detail::ArticleDetail as PaperArticleDetail;
use get_tv_article::ArticleDetail as TvArticleDetail;

use std::error::Error;

#[allow(dead_code)]
pub trait ExecSql<T> {
    async fn handle_post(
        _cfg: Extension<Arc<Config>>,
        _prms: Result<Json<T>, JsonRejection>,
    ) -> Result<Json<Value>, WebErr> {
        Ok(Json(json!({})))
    }

    async fn handle_post_with_token(
        _cfg: Extension<Arc<Config>>,
        _token: Extension<Arc<Mutex<MahTokenResp>>>,
        _prms: Result<Json<T>, JsonRejection>,
    ) -> Result<Json<Value>, WebErr> {
        Ok(Json(json!({})))
    }

    async fn handle_get(
        _cfg: Extension<Arc<Config>>,
        _prms: Option<Query<T>>,
    ) -> Result<Json<Value>, WebErr> {
        Ok(Json(json!({})))
    }
}

#[derive(Debug)]
pub struct WebErr(Box<dyn Error + Send + Sync>);

impl IntoResponse for WebErr {
    fn into_response(self) -> Response {
        let j = json!({
            "success": false,
            "errMsg": format!("{}", self.0),
            "data": "",
        })
        .to_string();
        Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(j.into())
            .unwrap()
    }
}

impl<E> From<E> for WebErr
where
    E: Into<Box<dyn Error + Send + Sync>>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

// --------------------

pub fn get_now_millis(extra: u32) -> u128 {
    let extra = extra as u128 * 1000;
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
        + extra
}

pub fn get_token(duration: u128) -> Result<String> {
    let salt = "ak9@j&pq1w(e423q";
    Ok(format!(
        "{:x}",
        md5::compute(format!("{},{}", salt, duration).as_bytes())
    ))
}

pub fn get_current_date() -> String {
    let n = Local::now();
    format!("{}-{:02}-{:02}", n.year(), n.month(), n.day(),)
}

pub fn get_yesterday() -> String {
    let n = Local::now();
    let t = n + ChDuration::days(-1);
    format!("{}-{:02}-{:02}", t.year(), t.month(), t.day())
}

pub fn get_tomorrow_date() -> String {
    let n = Local::now();
    let t = n + ChDuration::days(1);
    format!("{}-{:02}-{:02}", t.year(), t.month(), t.day())
}

fn timestamp_ms_to_local_date(timestamp_ms: i64, format: &str) -> Result<String> {
    // 毫秒转秒和纳秒（1毫秒=1000000纳秒）
    let sec = timestamp_ms / 1000;
    let nsec = (timestamp_ms % 1000) * 1_000_000;
    // 转换为本地时间
    let datetime = Local
        .timestamp_opt(sec, nsec as u32)
        .single()
        .ok_or(anyhow!("parse error"))?;
    Ok(datetime.format(format).to_string())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PostResponse {
    success: bool,
    err_msg: String,
    data: Value,
}

pub async fn post_tv_article(
    svr_url: &str,
    article: &TvArticleDetail,
    site_id: u32,
) -> Result<u32> {
    // let publish_date = timestamp_ms_to_local_date(article.show_date, "%Y-%m-%d")?;
    let publish_date_elements = article.show_date_str.split("-").collect::<Vec<_>>();
    let resp = reqwest::Client::new()
        .post(format!("{}/article_from_dump", svr_url))
        .json(&json!({
            "title": article.title,
            "tv_or_paper": site_id,
            "publish_year": publish_date_elements[0].parse::<u32>()?,
            "publish_month": publish_date_elements[1].parse::<u32>()?,
            "publish_day": publish_date_elements[2].parse::<u32>()?,
            "tv_url": "", // todo: 需要循环获取媒资地址
            "page_meta_id": 0,
            "page_name": "",
            "state": 1,
            // --
            "content": &article.content,
            "html_content": &article.html_content,
            "ref_id": article.id,
            "duration": article.video_time,
            "character_count": article.num_with_space
        }))
        .send()
        .await?;
    if resp.status() == StatusCode::OK {
        let txt = resp.text().await?;
        println!("txt is {}", &txt);
        let j = serde_json::from_str::<PostResponse>(&txt)?;
        if j.success {
            let id = j.data.as_i64().ok_or(anyhow!("invalid response"))?;
            Ok(id as u32)
        } else {
            Err(anyhow!("post tv article failed, {}", j.err_msg))
        }
    } else {
        Err(anyhow!("post tv article failed"))
    }
}

pub async fn post_tv_score(
    svr_url: &str,
    article_id: u32,
    article: &TvArticleDetail,
) -> Result<()> {
    let mut reporters = vec![];
    for code in article
        .author_codes
        .clone()
        .unwrap_or("".into())
        .trim_matches(',')
        .split(',')
        .collect::<Vec<&str>>()
        .into_iter()
    {
        reporters.push(json!({
            "ref_code": Some(code),
            "reporter_category_id": 3,
            "score": 0
        }));
    }
    for code in article
        .photo_person_codes
        .clone()
        .unwrap_or("".into())
        .trim_matches(',')
        .split(',')
        .collect::<Vec<&str>>()
        .into_iter()
    {
        reporters.push(json!({
            "reporter_id": null,
            "reporter_name": null,
            "reporter_category_id": 4,
            "ref_code": Some(code),
            "score": 0
        }));
    }
    let resp = reqwest::Client::new()
        .post(format!("{}/score_for_dump", svr_url))
        .json(&json!({
            "article_id": article_id,
            "score_basic": 0,
            "score_action": 0,
            "reporter_scores": reporters
        }))
        .send()
        .await?;
    if resp.status() == StatusCode::OK {
        let txt = resp.text().await?;
        println!("txt is {}", &txt);
        let j = serde_json::from_str::<PostResponse>(&txt)?;
        if j.success {
            let _id = j.data.as_i64().ok_or(anyhow!("invalid response"))?;
            Ok(())
        } else {
            Err(anyhow!("score tv article failed, {}", j.err_msg))
        }
    } else {
        let txt = resp.text().await?;
        println!("txt is {}", &txt);
        Err(anyhow!("score tv article failed"))
    }
}

pub async fn post_paper_article(
    svr_url: &str,
    article: &PaperArticleDetail,
    site_id: u32,
) -> Result<u32> {
    let paper_meta: HashMap<String, u32> = HashMap::from([
        ("一版".to_string(), 1),
        ("二版".to_string(), 2),
        ("三版".to_string(), 3),
        ("四版".to_string(), 4),
        ("五版".to_string(), 5),
        ("六版".to_string(), 6),
        ("七版".to_string(), 7),
        ("八版".to_string(), 8),
        ("九版".to_string(), 9),
        ("十版".to_string(), 10),
        ("十一版".to_string(), 11),
        ("十二版".to_string(), 12),
        ("十三版".to_string(), 13),
        ("十四版".to_string(), 14),
        ("十五版".to_string(), 15),
        ("十六版".to_string(), 16),
    ]);

    let publish_date_elements = article.pubdate.split("-").collect::<Vec<_>>();
    let page_meta_id = paper_meta.get(&article.chnldesc).unwrap_or(&0);
    let paper_url = get_article_pdf(
        publish_date_elements[0],
        publish_date_elements[1],
        publish_date_elements[2],
        *page_meta_id as usize,
    )
    .await
    .unwrap_or("".into());
    let resp = reqwest::Client::new()
        .post(format!("{}/article_from_dump", svr_url))
        .json(&json!({
            "title": article.title,
            "tv_or_paper": site_id,
            "publish_year": publish_date_elements[0].parse::<i32>()?,
            "publish_month": publish_date_elements[1].parse::<i32>()?,
            "publish_day": publish_date_elements[2].parse::<i32>()?,
            // "tv_url": "",
            "tv_url": paper_url, // 报纸版面的pdf文件
            "page_meta_id": page_meta_id,
            "page_name": &article.chnldesc,
            "state": 1,
            // --
            "content": article.content,
            "html_content": &article.htmlcontent,
            "ref_id": article.metadataid,
            "duration": 0,
            "character_count": article.docwordscount
        }))
        .send()
        .await?;
    if resp.status() == StatusCode::OK {
        let txt = resp.text().await?;
        println!("txt is {}", &txt);
        let j = serde_json::from_str::<PostResponse>(&txt)?;
        if j.success {
            let id = j.data.as_i64().ok_or(anyhow!("invalid response"))?;
            Ok(id as u32)
        } else {
            Err(anyhow!("post paper article failed, {}", j.err_msg))
        }
    } else {
        Err(anyhow!("post paper article failed"))
    }
}

pub async fn post_paper_score(
    svr_url: &str,
    article_id: u32,
    article: &PaperArticleDetail,
) -> Result<()> {
    let resp = reqwest::Client::new()
        .post(format!("{}/score", svr_url))
        .json(&json!({
            "article_id": article_id,
            "score_basic": 0,
            "score_action": 0,
            "reporter_scores": vec![json!({
                "reporter_name": &article.author,
                "reporter_category_id": 4,
                "score": 0,
            })]
        }))
        .send()
        .await?;
    if resp.status() == StatusCode::OK {
        let txt = resp.text().await?;
        println!("txt is {}", &txt);
        let j = serde_json::from_str::<PostResponse>(&txt)?;
        if j.success {
            let _id = j.data.as_i64().ok_or(anyhow!("invalid response"))?;
            Ok(())
        } else {
            Err(anyhow!("score tv article failed, {}", j.err_msg))
        }
    } else {
        Err(anyhow!("score tv article failed"))
    }
}

async fn get_article_pdf(
    year: &str,
    month: &str,
    day: &str,
    page_meta_id: usize,
) -> Result<String> {
    let url = format!(
        "https://epaper.wifizs.cn/zsrb/{}-{}/{}/node_{}.html",
        year, month, day, page_meta_id
    );
    let resp = reqwest::get(url).await?;
    let txt = resp.text().await?;
    let html = Html::parse_document(&txt);
    let selector = Selector::parse("a.pdf").unwrap();
    for (index, element) in html.select(&selector).into_iter().enumerate() {
        if index == page_meta_id - 1 {
            let href = match element.value().attr("href") {
                Some(t) => t.to_string(),
                None => return Err(anyhow!("invalid response")),
            };
            return Ok(href);
        }
    }
    Err(anyhow!("invalid response"))
}
