use std::net::SocketAddr;

use axum::{response::Html, routing::get, Router};
use indoc::indoc;

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(root));

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .expect("Could not parse port");

    // run our app with hyper
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    println!("listening on http://{}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}

// basic handler that responds with a static string
async fn root() -> Html<&'static str> {
    Html(indoc! {
        "
        <!DOCTYPE html>
        <html>
            <head>
                <title>Mario!</title>
            </head>
            <body>
                <h1>Mario!</h1>
                <p>It's a me, Mario!</p>
            </body>
        </html>
        "
    })
}
