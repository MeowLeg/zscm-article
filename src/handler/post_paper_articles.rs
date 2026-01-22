use super::*;

use crate::handler::get_paper_article_detail::get_paper_article_detail;
use crate::handler::get_paper_articles::get_paper_articles;

pub struct PostPaperArticles;

#[derive(Debug, Deserialize, Default)]
pub struct PostPaperArticlesReq {
    site_id: Option<u32>,
}

impl ExecSql<PostPaperArticlesReq> for PostPaperArticles {
    async fn handle_get(
        cfg: Extension<Arc<Config>>,
        prms: Option<Query<PostPaperArticlesReq>>,
    ) -> Result<Json<Value>, WebErr> {
        println!("rth-0");
        // let cur_date = get_current_date();
        let cur_date = get_yesterday();
        let Query(prms) = prms.unwrap_or_default();
        let site_id = prms.site_id.unwrap_or(1);
        match get_paper_articles(
            &cfg.paper_server_url,
            cfg.site_id,
            Some(cfg.docstatus),
            Some(cur_date.clone()),
            Some(cur_date.clone()),
            cfg.timestamp_extra,
        )
        .await
        {
            Ok(data) => {
                for article in data {
                    let metadata_id = article.metadataid;
                    match get_paper_article_detail(
                        &cfg.paper_server_url,
                        metadata_id,
                        cfg.timestamp_extra,
                    )
                    .await
                    {
                        Ok(detail) => {
                            // println!("{:?}", &detail);
                            match post_paper_article(&cfg.post_server_url, &detail, site_id).await {
                                Ok(a_id) => {
                                    match post_paper_score(&cfg.post_server_url, a_id, &detail)
                                        .await
                                    {
                                        Ok(_) => {
                                            println!("post_paper_score success");
                                        }
                                        Err(err) => {
                                            println!("post_paper_score failed: {}", err);
                                        }
                                    }
                                }
                                Err(err) => {
                                    println!("post_paper_article failed: {}", err);
                                }
                            }
                        }
                        Err(err) => {
                            println!("get_paper_article_detail failed: {}", err);
                        }
                    }
                }
            }
            Err(err) => {
                println!("get_paper_articles failed: {}", err);
            }
        }
        Ok(Json(json!({
            "success": true,
            "errMsg": "",
            "data": Value::Null
        })))
    }
}
