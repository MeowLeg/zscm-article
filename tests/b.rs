use base64::prelude::*;

#[test]
fn test_base64() {
    let b = BASE64_STANDARD.encode(format!(
        "{}:{}",
        "zcsmwages", "b79dcbd53aea46e3acc7887aae72f5f0"
    ));
    println!("base64: {}", b);
    assert!(true);
}

#[test]
fn test_reg() {
    use regex::Regex;
    // let title = "舟山城市足球联赛（县区组）启幕 百余名球员绿茵争锋 （洪碧 刘中华）";
    let title = "（每日快讯）舟山海警局查获一起走私成品油案（通讯员：虞康铭、司马志潮）";
    let reg = Regex::new("（.*?）").unwrap();
    let new_title = reg.replace_all(&title, " ");
    let trim_new_title = new_title.trim().to_string();
    println!("new_title: <{}>", trim_new_title);
    assert!(true);
}

#[tokio::test]
async fn test_scraper() {
    let url = "https://epaper.wifizs.cn/zsrb/2026-01/23/node_1.html";
    let resp = reqwest::get(url).await.unwrap();
    let txt = resp.text().await.unwrap();
    use scraper::{Html, Selector};
    let html = Html::parse_document(&txt);
    let selector = Selector::parse("a.pdf").unwrap();
    for element in html.select(&selector) {
        let href = element.value().attr("href").unwrap();
        println!("href: {}", href);
    }
    assert!(true);
}
