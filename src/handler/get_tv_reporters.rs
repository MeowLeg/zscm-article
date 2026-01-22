use super::*;

pub struct GetTvReporters;

#[derive(Deserialize)]
pub struct GetTvReportersReq;

#[derive(Debug, Serialize, Deserialize)]
pub struct TvReporterResp {
    message: String,
    data: Vec<(u32, String, String)>,
}

impl ExecSql<GetTvReportersReq> for GetTvReporters {
    async fn handle_get(
        cfg: Extension<Arc<Config>>,
        _prms: Option<Query<GetTvReportersReq>>,
    ) -> Result<Json<Value>, WebErr> {
        let sql = r#"
            select id, nickname, usercode from sh_user
            "#;
        let url = format!("{}/s/doc/queryBySQL", cfg.tv_server_url);
        println!("URL: {}", &url);
        let cli = reqwest::Client::new();
        let resp = cli
            .post(&url)
            .json(&json!({
                "SQL": sql,
            }))
            .send()
            .await?;
        if resp.status() == StatusCode::OK {
            let txt = resp.text().await?;
            println!("{}", &txt);
            let data = serde_json::from_str::<TvReporterResp>(&txt)?;
            return Ok(Json(json!({
                "success": true,
                "errMsg": "",
                "data": data.data
            })));
        } else {
            let txt = resp.text().await?;
            println!("Error: {}", &txt);
        }
        Ok(Json(json!({
            "success": false,
            "errMsg": "",
            "data": Value::Null
        })))
    }
}
