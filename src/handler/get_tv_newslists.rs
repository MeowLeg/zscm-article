use core::str;

use super::*;

pub struct GetTvNewsLists;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetTvNewsListsReq {
    pub column_id: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TvNewsList {
    pub id: String,
    pub title: String,
    pub showdate: String,
    pub columnid: String,
    pub column_code: String,
    pub column_name: String,
    pub channel_id: String,
    pub time_period_id: Option<String>,
    pub time_period_name: Option<String>,
    pub editorby: String,
    pub editorbycode: String,
}

impl ExecSql<GetTvNewsListsReq> for GetTvNewsLists {
    async fn handle_get(
        cfg: Extension<Arc<Config>>,
        prms: Option<Query<GetTvNewsListsReq>>,
    ) -> Result<Json<Value>, WebErr> {
        let Query(prms) = prms.ok_or_else(|| WebErr("prms is None".into()))?;
        let column_id = prms.column_id.clone().unwrap_or("".into());
        let resp = get_tv_newslists(
            &cfg.tv_server_url,
            &column_id,
            prms.start_time.clone(),
            prms.end_time.clone(),
        )
        .await?;
        Ok(Json(json!({
            "success": true,
            "errMsg": "",
            "data": resp,
        })))
    }
}

pub async fn get_tv_newslists(
    server_url: &str,
    colummnid: &str,
    start_time: Option<String>,
    end_time: Option<String>,
) -> Result<Vec<TvNewsList>> {
    println!("column_id: {}", colummnid);
    let cur_date = get_current_date();
    let tomorrow = get_tomorrow_date();
    let start_time = start_time.unwrap_or(format!("{}%2000:00:00", &cur_date));
    let end_time = end_time.unwrap_or(format!("{}%2000:00:00", &tomorrow));
    let url = format!(
        "{}/s/llist/getLlistByconditionsQuery?columnid={}&startTime={}&endTime={}",
        server_url, colummnid, start_time, end_time
    );
    println!("url: {}", &url);
    let resp = reqwest::Client::new().get(url).send().await?;
    // println!("resp: {:?}", &resp);
    let txt = resp.text().await?;
    // println!("txt is {}", &txt);
    let d = serde_json::from_str::<Vec<TvNewsList>>(&txt)?;
    // println!("d: {:?}", &d);
    Ok(d)
}
