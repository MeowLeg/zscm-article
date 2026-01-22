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
