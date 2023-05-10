use http::{MediaType, ResponseContentType, HTTP_CLIENT};

// #[tokio::main]
fn main() {
    let res = HTTP_CLIENT
        .get("http://127.0.0.1:3000")
        .send()
        .expect("Could not send request");

    println!("Status: {}", res.status());
    println!("Headers: {:?}", res.headers());

    assert_eq!(res.status(), http::StatusCode::OK);
    assert_eq!(
        *res.content_type().unwrap().media_type(),
        MediaType::TextHTML
    );

    // println!("Body: \n{:?}", res.text());

    let _ = html::HtmlParser::new().try_parse(res);
}
