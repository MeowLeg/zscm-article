use super::*;
use base64::prelude::*;
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};

pub struct GetMahToken;

#[derive(Debug, Deserialize)]
pub struct GetMahTokenReq {
    username: Option<String>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct MahTokenResp {
    pub access_token: String,
    token_type: String,
    refresh_token: String,
    expires_in: u64,
    scope: String,
    jti: String,
}

impl ExecSql<GetMahTokenReq> for GetMahToken {
    async fn handle_get(
        cfg: Extension<Arc<Config>>,
        prms: Option<Query<GetMahTokenReq>>,
    ) -> Result<Json<Value>, WebErr> {
        let Query(prms) = prms.ok_or("Missing parameters")?;
        let username = match prms.username {
            Some(un) => un,
            None => "".into(),
        };
        // let auth = get_auth(&cfg.mah_client_id, &cfg.mah_client_secret)?;
        // println!("username: {}, auth: {}", &username, &auth);
        let token =
            // get_mah_token(&cfg.mah_token_server_url, &username, &auth, &cfg.grant_type).await?;
            get_mah_token(&cfg.mah_token_server_url, &username, &cfg.mah_client_id, &cfg.mah_client_secret, &cfg.grant_type).await?;
        Ok(Json(json!({
            "success": true,
            "errMsg": "获取token成功",
            "data": token
        })))
    }
}

async fn get_mah_token(
    url: &str,
    username: &str,
    // auth: &str,
    client_id: &str,
    client_secret: &str,
    grant_type: &str,
) -> Result<MahTokenResp> {
    let url = format!("{}/oauth/token", url);
    println!("url is {}", &url);
    let username = utf8_percent_encode(username, NON_ALPHANUMERIC).to_string();
    let mut form_data = HashMap::new();
    form_data.insert("grant_type", grant_type);
    form_data.insert("username", &username);
    // let auth = format!("Basic {{Base64({}:{})}}", client_id, client_secret);
    let auth = format!("Basic {}", get_auth(&client_id, &client_secret)?);
    println!("auth is {}", &auth);
    let cli = reqwest::Client::builder()
        .no_proxy()
        .danger_accept_invalid_hostnames(true)
        .danger_accept_invalid_certs(true)
        .build()?;
    let resp = cli
        .post(&url)
        .form(&form_data)
        .header(reqwest::header::AUTHORIZATION, &auth)
        .header(reqwest::header::HOST, "auth.wifizs.cn")
        // .header(reqwest::header::AUTHORIZATION, &format!("Basic {}", &auth))
        // .header("sobeycloud-parent-user", "admin")
        .send()
        .await?;
    if resp.status() == StatusCode::OK {
        let txt = resp.text().await?;
        println!("txt is {}", &txt);
        let token: MahTokenResp = serde_json::from_str(&txt)?;
        Ok(token)
    } else {
        let txt = resp.text().await?;
        println!("txt is {}", &txt);
        Err(anyhow!("获取token失败"))
    }
}

fn get_auth(client_id: &str, client_secret: &str) -> Result<String> {
    let ori_data = format!("{}:{}", client_id, client_secret);
    Ok(BASE64_STANDARD.encode(ori_data))
}

pub async fn loop_get_access_token(
    cfg: Arc<Config>,
    sobey_token: Arc<Mutex<MahTokenResp>>,
) -> Result<()> {
    let st = Arc::clone(&sobey_token);
    let auth = get_auth(&cfg.mah_client_id, &cfg.mah_client_secret)?;
    loop {
        let is_expired = {
            match st.lock() {
                Ok(t) => t.expires_in <= cfg.loop_token_interval,
                Err(_) => false,
            }
        };
        if is_expired {
            let t1 = get_mah_token(
                &cfg.mah_token_server_url,
                &cfg.mah_username,
                // &auth,
                &cfg.mah_client_id,
                &cfg.mah_client_secret,
                &cfg.grant_type,
            )
            .await
            .unwrap_or(MahTokenResp::default());
            {
                match st.lock() {
                    Ok(mut t) => {
                        println!("token is {:?}", &t1);
                        *t = MahTokenResp { ..t1 };
                    }
                    Err(_) => {}
                }
            }
        } else {
            match st.lock() {
                Ok(mut t) => {
                    t.expires_in -= cfg.loop_token_interval + 5; // 为了避免超出
                }
                Err(_) => {}
            }
        }

        sleep(Duration::from_secs(cfg.loop_token_interval)).await;
    }
}
