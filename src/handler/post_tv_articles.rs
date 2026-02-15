use super::*;

use crate::handler::get_tv_article::get_tv_article;
use crate::handler::get_tv_newslist_detail::{get_tv_newslist_detail, get_tv_newslist_docids};
use crate::handler::get_tv_newslists::get_tv_newslists;

pub struct PostTvArticles;

#[derive(Debug, Deserialize, Default)]
pub struct PostTvArticlesReq {
    column_id: Option<String>,
    site_id: Option<u32>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
}

impl ExecSql<PostTvArticlesReq> for PostTvArticles {
    async fn handle_get(
        cfg: Extension<Arc<Config>>,
        prms: Option<Query<PostTvArticlesReq>>,
    ) -> Result<Json<Value>, WebErr> {
        let Query(prms) = prms.ok_or("Missing parameters")?;
        let column_id = prms.column_id.clone().unwrap_or(cfg.tv_columnid.clone());
        let site_id = prms.site_id.unwrap_or(0);
        match get_tv_newslists(
            &cfg.tv_server_url,
            &column_id,
            prms.start_time,
            prms.end_time,
        )
        .await
        {
            Ok(data) => {
                // println!("newslists: {:?}", &data);
                for lst in data.iter() {
                    let llistid = &lst.id;
                    let showdates: Vec<&str> = lst.showdate.split(' ').collect();
                    match get_tv_newslist_detail(&cfg.tv_server_url, llistid).await {
                        Ok(detail) => {
                            // println!("detail: {:?}", &detail);
                            let ids = get_tv_newslist_docids(&detail);
                            // println!("ids: {:?}", &ids);
                            for id_inf in ids.into_iter() {
                                match get_tv_article(&cfg.tv_server_url, id_inf.0).await {
                                    Ok(mut article) => {
                                        if filter_title(&article.title, &cfg.filter_words)? {
                                            article.video_time = id_inf.1;
                                            article.show_date_str = String::from(showdates[0]);
                                            // println!("{:?}", &article);
                                            match post_tv_article(
                                                &cfg.post_server_url,
                                                &article,
                                                site_id,
                                            )
                                            .await
                                            {
                                                Ok(a_id) => {
                                                    match post_tv_score(
                                                        &cfg.post_server_url,
                                                        a_id,
                                                        &article,
                                                    )
                                                    .await
                                                    {
                                                        Ok(()) => {
                                                            println!("post_tv_score Success");
                                                        }
                                                        Err(e) => {
                                                            println!("post_tv_score Error: {}", e);
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    println!("post_tv_article Error: {}", e);
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        println!("get_tv_article Error: {}", e);
                                        println!("article is {:?}", &id_inf);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            println!("get_tv_newslist_detail Error: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                println!("get_tv_newslists Error: {}", e);
            }
        }
        Ok(Json(json!({
            "success": true,
            "errMsg": "",
            "data": {},
        })))
    }
}

fn filter_title(title: &str, filter_words: &[String]) -> Result<bool> {
    for word in filter_words {
        let reg = Regex::new(word)?;
        if reg.is_match(title) {
            return Ok(false);
        }
    }
    Ok(true)
}
