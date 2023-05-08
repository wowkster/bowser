use http::{MediaType, ResponseContentType, HTTP_CLIENT};

#[tokio::main]
async fn main() {
    let res = HTTP_CLIENT
        .get("http://127.0.0.1:3000")
        .send()
        .await
        .expect("Could not send request");

    println!("Status: {}", res.status());
    println!("Headers: {:?}", res.headers());

    assert_eq!(res.status(), http::StatusCode::OK);
    assert_eq!(
        *res.content_type().unwrap().media_type(),
        MediaType::TextHTML
    );

    let body = res.text().await.expect("Could not read response body");

    println!("Body: \n{}", body);

    html::parse_string(body);
}
