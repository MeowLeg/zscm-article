use super::*;

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
    pub begin_date: Option<String>,
    pub end_date: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct GetArticlesResp {
    success: bool,
    message: String,
    data: Vec<Article>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub struct Article {
    pub metadataid: u32,
    pub doctype: u32,
    pub title: String,
    pub pubdate: String,
    pub author: String,
    pub chnldesc: String,
    pub docstatus: u32,
}

impl ExecSql<GetArticlesReq> for GetArticles {
    async fn handle_get(
        cfg: Extension<Arc<Config>>,
        prms: Option<Query<GetArticlesReq>>,
    ) -> Result<Json<Value>, WebErr> {
        let Query(prms) = prms.ok_or(anyhow!("Missing Parameters"))?;
        let data = get_paper_articles(
            &cfg.paper_server_url,
            prms.site_id,
            prms.docstatus,
            prms.begin_date,
            prms.end_date,
            cfg.timestamp_extra,
        )
        .await?;
        Ok(Json(json!({
            "success": true,
            "errMsg": "获取文章",
            "data": data
        })))
    }
}

pub async fn get_paper_articles(
    server_url: &str,
    site_id: u32,
    docstatus: Option<u32>,
    begin_date: Option<String>,
    end_date: Option<String>,
    extra: u32,
) -> Result<Vec<Article>> {
    let now_millis = get_now_millis(extra);
    let token = get_token(now_millis)?;
    let c_date = get_current_date();
    let url = format!(
        "{}/paper/docs?token={}&timestamp={}&siteId={}&beginDate={}&endDate={}&docstatus={}",
        // "{}/paper/docs?token={}&timestamp={}&siteId={}&beginDate={}&endDate={}",
        server_url,
        token,
        now_millis,
        site_id,
        begin_date.unwrap_or(c_date.clone()),
        end_date.unwrap_or(c_date),
        docstatus.unwrap_or(38)
    );
    println!("url is {}", &url);
    let resp = reqwest::get(&url).await?;
    let txt = resp.text().await?;
    println!("{}", &txt);
    let data: GetArticlesResp = serde_json::from_str(&txt)?;
    Ok(data.data)
}
