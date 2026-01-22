use super::*;

pub struct GetArticleDetail;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetArticleDetailReq {
    pub metadata_id: u32,
}

#[derive(Debug, Deserialize, Serialize)]
struct GetArticleDetailResp {
    success: bool,
    message: String,
    data: ArticleDetail,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub struct ArticleDetail {
    pub metadataid: u32,
    pub doctype: u32,
    pub title: String,
    pub pubdate: String,
    pub author: String,
    pub chnldesc: String,
    pub docstatus: u32,
    pub docwordscount: u32,
    pub htmlcontent: String,
    pub content: String,
    pub remarks: String,
}

impl ExecSql<GetArticleDetailReq> for GetArticleDetail {
    async fn handle_get(
        cfg: Extension<Arc<Config>>,
        prms: Option<Query<GetArticleDetailReq>>,
    ) -> Result<Json<Value>, WebErr> {
        let prms = prms.ok_or("Missing parameters")?;
        let data =
            get_paper_article_detail(&cfg.paper_server_url, prms.metadata_id, cfg.timestamp_extra)
                .await?;
        Ok(Json(json!({
            "success": true,
            "errMsg": "获取文章详情",
            "data": data
        })))
    }
}

pub async fn get_paper_article_detail(
    server_url: &str,
    metadata_id: u32,
    extra: u32,
) -> Result<ArticleDetail> {
    let mis = get_now_millis(extra);
    let resp = reqwest::get(format!(
        "{}/paper/docDetail?metadataId={}&timestamp={}&token={}",
        server_url,
        metadata_id,
        mis,
        get_token(mis)?,
    ))
    .await?;
    let txt = resp.text().await?;
    // println!("{}", &txt);
    let data: GetArticleDetailResp = serde_json::from_str(&txt)?;
    Ok(data.data)
}
