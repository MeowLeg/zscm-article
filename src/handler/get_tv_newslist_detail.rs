use super::*;

pub struct GetTvNewsListDetail;

#[derive(Debug, Deserialize)]
pub struct GetTvNewsListDetailReq {
    pub llistid: String,
}

impl ExecSql<GetTvNewsListDetailReq> for GetTvNewsListDetail {
    async fn handle_get(
        cfg: Extension<Arc<Config>>,
        prms: Option<Query<GetTvNewsListDetailReq>>,
    ) -> Result<Json<Value>, WebErr> {
        let prms = prms.ok_or("Missing parameters")?;
        let v = get_tv_newslist_detail(&cfg.tv_server_url, &prms.llistid).await?;
        let ids = get_tv_newslist_docids(&v);
        Ok(Json(json!({
            "success": true,
            "errMsg": "",
            "data": {
                "detail": v,
                "docids": ids,
            }
        })))
    }
}

pub async fn get_tv_newslist_detail(server_url: &str, llistid: &str) -> Result<Value> {
    let url = &format!("{}/s/llist/getLlistno/{}", server_url, llistid);
    println!("url is {}", &url);
    let resp = reqwest::Client::new()
        .get(url)
        .send()
        .await?
        .json::<Value>()
        .await?;
    Ok(resp)
}

fn video_time_to_u64(video_time: &str) -> u64 {
    let vs = video_time.split("'").collect::<Vec<&str>>();
    if vs.len() < 3 {
        return 0;
    }
    let h = vs[0].parse::<u64>().unwrap_or(0);
    let m = vs[1].parse::<u64>().unwrap_or(0);
    let s = vs[2].parse::<u64>().unwrap_or(0);
    h * 3600 + m * 60 + s
}

pub fn get_tv_newslist_docids(resp: &Value) -> Vec<(u64, u64)> {
    let mut docids = Vec::new();
    if let Some(list) = resp["list"].as_object() {
        if let Some(parent_list) = list["parentList"].as_array() {
            for item in parent_list {
                if let Some(reldocid) = item["pgmmaster"]["reldocid"].as_u64()
                    && reldocid != 0
                {
                    // 未被删除
                    if let Some(deleteflag) = item["doc"]["deleteflag"].as_i64()
                        && deleteflag == 0
                    {
                        if let Some(materialguid) = item["pgmmaster"]["materialguid"].as_str()
                            && materialguid != ""
                        {
                            if let Some(video_time) = item["videoTime"].as_str() {
                                docids.push((reldocid, video_time_to_u64(video_time)));
                            }
                        }
                    }
                }
            }
        }
    }
    // println!("docids: {:?}", &docids);
    docids
}
