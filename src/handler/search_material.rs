use super::*;

pub struct SearchMaterail;

#[derive(Debug, Deserialize)]
pub struct SearchMaterialReq {
    title: String,
    page: Option<u32>,
    page_size: Option<u32>,
}

impl ExecSql<SearchMaterialReq> for SearchMaterail {
    async fn handle_post_with_token(
        Extension(cfg): Extension<Arc<Config>>,
        token: Extension<Arc<Mutex<MahTokenResp>>>,
        prms: Result<Json<SearchMaterialReq>, JsonRejection>,
    ) -> Result<Json<Value>, WebErr> {
        let Json(prms) = prms?;
        if prms.title.len() < cfg.search_char_at_least {
            return Err(format!(
                "search cahracters length should more than {}",
                cfg.search_char_at_least
            )
            .into());
        }
        let token = {
            let tk = Arc::clone(&token);
            match tk.lock() {
                Ok(t) => t.access_token.clone(),
                Err(_) => "".into(),
            }
        };
        let resp = search_article(
            &prms.title,
            prms.page.unwrap_or(1),
            prms.page_size.unwrap_or(10),
            &cfg.mah_search_server_url,
            &cfg.mah_content_server_url,
            &token,
        )
        .await?;
        return Ok(Json(json!({
            "success": true,
            "errMsg": "获取查询成功",
            "data": resp
        })));
    }
}

#[derive(Debug, Serialize, Default)]
struct MaterialInfo {
    // #[serde(rename = "name_")]
    name: String,
    // #[serde(rename = "contentId_")]
    content_id: String,
    // #[serde(rename = "keyframepath_")]
    key_frame_path: String,
    file_paths: Vec<String>,
}

fn title_refactor(title: &str) -> Result<String> {
    let reg = Regex::new("（.*?）")?;
    let new_title = reg.replace_all(&title, " ");
    let trim_new_title = new_title.trim().to_string();
    Ok(trim_new_title)
}

async fn search_article(
    // prms: &SearchMaterialReq,
    title: &str,
    page: u32,
    page_size: u32,
    mah_search_server_url: &str,
    mah_content_server_url: &str,
    token: &str,
) -> Result<Vec<MaterialInfo>> {
    let cli = reqwest::Client::builder()
        .no_proxy()
        .danger_accept_invalid_hostnames(true)
        .danger_accept_invalid_certs(true)
        .build()?;
    let url = format!("{}/v1/search", &mah_search_server_url);
    // println!("url is {}", &url);
    let resp = match cli
        .post(&url)
        .json(&json!({
            "pageIndex": page,
            "pageSize": page_size,
            "personId": "",
            "keywords": [
                title_refactor(title)?,
            ],
            "facetConditions": [],
            "conditions": [],
            "sortFields": [
                {
                    "field": "_score",
                    "isDesc": true
                }
            ],
            "isQueryDirectSub": true,
            "keywordSearchType": "fullText"
        }))
        .header("sobeycloud-token", token)
        .header(reqwest::header::HOST, "mah.wifizs.cn")
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(err) => {
            println!("Error sending request: {}", err);
            return Err(err.into());
        }
    };
    if resp.status() == StatusCode::OK {
        let txt = resp.text().await?;
        // println!("search result is {}", &txt);
        let v: Value = serde_json::from_str(&txt)?;
        return get_material_info(mah_content_server_url, &v, token).await;
    } else {
        let txt = resp.text().await?;
        println!("search result is {}", &txt);
        return Err(anyhow!("can not get search result"));
    }
}

async fn get_material_info(url: &str, v: &Value, token: &str) -> Result<Vec<MaterialInfo>> {
    let mut res: Vec<MaterialInfo> = vec![];
    if let Some(b) = v["success"].as_bool()
        && b
    {
        if let Some(data) = v["data"]["data"].as_array() {
            for d in data.into_iter() {
                if let Some(nm) = d["name_"].as_str() {
                    let mut mi = MaterialInfo::default();
                    mi.name = nm.to_string();
                    if let Some(kf) = d["keyframepath_"].as_str() {
                        mi.key_frame_path = kf.to_string();
                        if let Some(ci) = d["contentId_"].as_str() {
                            mi.content_id = ci.to_string();
                            let files = get_material_path(url, ci, token).await.unwrap_or(vec![]);
                            mi.file_paths = files;
                            res.push(mi);
                        }
                    }
                }
            }
        }
    }
    Ok(res)
}

async fn get_material_path(url: &str, content_id: &str, token: &str) -> Result<Vec<String>> {
    let cli = reqwest::Client::builder()
        .no_proxy()
        .danger_accept_invalid_hostnames(true)
        .danger_accept_invalid_certs(true)
        .build()?;
    let resp = match cli
        .get(format!("{}/v2/entity/base/{}", url, content_id))
        .header("sobeycloud-token", token)
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(err) => {
            println!("error getting material path: {}", err);
            return Err(err.into());
        }
    };
    println!("resp: {:?}", &resp);
    if resp.status() == StatusCode::OK {
        let txt = resp.text().await?;
        println!("entity result is {}", &txt);
        let v: Value = serde_json::from_str(&txt)?;
        if let Some(b) = v["success"].as_bool()
            && b
        {
            if let Some(pv) = v["data"]["previewFile"].as_array() {
                return Ok(pv
                    .into_iter()
                    .map(|v| v["filePath"].as_str().unwrap_or("").to_string())
                    .collect());
            } else {
                return Err(anyhow!("can not get data previewFile"));
            }
        } else {
            return Err(anyhow!("not success"));
        }
    } else {
        let txt = resp.text().await?;
        println!("can not get filepathes: {}", txt);
        Err(anyhow!("can not get filepathes"))
    }
}

pub async fn loop_get_tv_url(
    year: Option<u32>,
    month: Option<u32>,
    mah_search_server_url: String,
    mah_content_server_url: String,
    post_server_url: String,
    loop_tv_url_interval: u64,
    token: Arc<Mutex<MahTokenResp>>,
) -> Result<()> {
    loop {
        sleep(Duration::from_secs(3)).await;

        let today_str = get_current_date();
        let ts = today_str.split('-').collect::<Vec<&str>>();
        let year = year.unwrap_or(ts[0].parse().unwrap_or(0));
        let month = month.unwrap_or(ts[1].parse().unwrap_or(0));
        let artiles_url = format!(
            "{}/get_articles?page=1&limit=20&tv_url=&tv_or_paper=0,1&year={}&month={}",
            &post_server_url, year, month
        );
        let resp = reqwest::get(artiles_url).await?;
        println!("loop article resp is {:?}", &resp);
        let token = {
            let tk = Arc::clone(&token);
            match tk.lock() {
                Ok(t) => t.access_token.clone(),
                Err(_) => "".into(),
            }
        };
        if resp.status() == StatusCode::OK {
            let mut id_mat: Vec<(u64, String)> = Vec::new();
            let txt = resp.text().await?;
            // println!("loop article txt is {:?}", &txt);
            let resp_data: Value = serde_json::from_str(&txt)?;
            if resp_data["success"].as_bool().unwrap_or(false) {
                println!("rth-1");
                if let Some(articles) = resp_data["data"]["articles"].as_array() {
                    println!("rth-2");
                    for item in articles {
                        println!("rth-3");
                        if let Some(title) = item["title"].as_str() {
                            let resp = search_article(
                                title,
                                1,
                                20,
                                &mah_search_server_url,
                                &mah_content_server_url,
                                &token,
                            )
                            .await?;
                            println!("search article resp is {:?}", &resp);
                            if resp.len() > 0 {
                                if resp[0].file_paths.len() > 0 {
                                    id_mat.push((
                                        item["id"].as_u64().unwrap_or(0),
                                        resp[0].file_paths[0].clone(),
                                    ));
                                }
                            }
                        }
                    }
                }
            }
            let cli = reqwest::Client::new();
            let _resp = cli
                .post(format!("{}/batch_update_tv_urls", post_server_url))
                .json(&json!({
                    "tv_urls": id_mat
                }))
                .send()
                .await;
        }

        sleep(Duration::from_secs(loop_tv_url_interval)).await;
    }
}
