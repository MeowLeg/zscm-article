use super::*;

pub struct GetTvArticle;

#[derive(Debug, Deserialize)]
pub struct GetTvArticleReq {
    docid: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ArticleDetail {
    pub id: u64,
    pub title: String,
    pub content: String,
    #[serde(rename = "htmlcontext")]
    pub html_content: String,
    pub author: String,
    #[serde(rename = "authorcode")]
    pub author_codes: String,
    #[serde(rename = "photoperson")]
    pub photo_person: Option<String>,
    #[serde(rename = "photopersoncode")]
    pub photo_person_codes: Option<String>,
    #[serde(rename = "updatedBy")]
    pub updated_by: String,
    #[serde(rename = "columnname")]
    pub colunm_name: String,
    #[serde(rename = "showdate")]
    pub show_date: i64,
    #[serde(rename = "numWithoutSpace")]
    pub num_without_space: u32,
    #[serde(rename = "numWithSpace")]
    pub num_with_space: u32,
    #[serde(default)]
    pub video_time: u64,
    #[serde(default)]
    pub show_date_str: String,
}

impl ExecSql<GetTvArticleReq> for GetTvArticle {
    async fn handle_get(
        cfg: Extension<Arc<Config>>,
        prms: Option<Query<GetTvArticleReq>>,
    ) -> Result<Json<Value>, WebErr> {
        let prms = prms.ok_or("Missing parameters")?;
        let v = get_tv_article(&cfg.tv_server_url, prms.docid).await?;
        Ok(Json(json!({
            "success": true,
            "errMsg": "",
            "data": v,
        })))
    }
}

pub async fn get_tv_article(server_url: &str, docid: u64) -> Result<ArticleDetail> {
    let url = format!("{}/s/doc/{}", server_url, docid);
    println!("url: {}", &url);
    let resp = reqwest::get(url).await?;
    let txt = resp.text().await?;
    // println!("txt: {}", &txt);
    match serde_json::from_str::<ArticleDetail>(&txt) {
        Ok(v) => Ok(v),
        Err(e) => {
            println!("txt is {}", txt);
            Err(anyhow!("Failed to parse JSON: {}", e))
        }
    }
}
