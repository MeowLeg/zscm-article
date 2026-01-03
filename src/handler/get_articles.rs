use clap::builder::Str;
use reqwest::StatusCode;

use super::*;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct GetArticles;

// fn default_docstatus() -> u32 {
//     30
// }

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetArticlesReq {
    pub site_id: u32,
    // #[serde(default = "default_docstatus")]
    pub docstatus: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
struct GetArticlesResp {
    success: bool,
    message: String,
    data: Vec<Article>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
struct Article {
    metadataid: u32,
    doctype: u32,
    title: String,
    pubdate: String,
    author: String,
    chnldesc: String,
    docstatus: u32,
}

impl ExecSql<GetArticlesReq> for GetArticles {
    async fn handle_get(
        cfg: Extension<Arc<Config>>,
        prms: Option<Query<GetArticlesReq>>,
    ) -> Result<Json<Value>> {
        let Query(prms) = prms.ok_or(anyhow!("Missing Parameters"))?;
        let now_millis = get_now_millis();
        let token = get_token(now_millis)?;
        let c_date = get_current_date();
        let url = format!(
            "{}/paper/docs?token={}&timestamp={}&siteId={}&docstatus={}&BeginDate={}&EndDate={}",
            &cfg.server_url,
            token,
            now_millis,
            prms.site_id,
            prms.docstatus.unwrap_or(30),
            c_date,
            c_date
        );
        let cli = reqwest::Client::new();
        let resp: GetArticlesResp = cli
            .get(&url)
            .send()
            .await?
            .json::<GetArticlesResp>()
            .await?;
        Ok(Json(json!({
            "success": resp.success,
            "errMsg": resp.message,
            "data": resp.data
        })))
    }
}

fn get_now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

fn get_token(duration: u128) -> Result<String> {
    let salt = "ak9@j&pq1w(e423q";
    Ok(format!(
        "{:x}",
        md5::compute(format!("{}{}", salt, duration).as_bytes())
    ))
}

fn get_current_date() -> String {
    let n = Local::now();
    format!("{}-{:02}-{:02}", n.year(), n.month(), n.day(),)
}
