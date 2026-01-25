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
    let title = "舟山城市足球联赛（县区组）启幕 百余名球员绿茵争锋 （洪碧 刘中华）";
    let reg = Regex::new("（.*?）").unwrap();
    let new_title = reg.replace_all(&title, " ");
    let trim_new_title = new_title.trim().to_string();
    println!("new_title: {}", trim_new_title);
    assert!(true);
}
