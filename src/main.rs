use actix_web::{web, App, HttpResponse, HttpServer, Responder, get};
use serde::Deserialize;
use reqwest::Client;
use chrono::NaiveDateTime;

#[derive(Deserialize)]
struct QueryParams {
    query: String,
}

#[get("/")]
async fn index() -> impl Responder {
    let html = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <meta charset="UTF-8">
            <title>Crypto News Aggregator</title>
        </head>
        <body>
            <h1>Поиск криптовалютных новостей</h1>
            <form action="/news" method="get">
                <label for="query">Введите символ криптовалюты (например, BTC):</label><br>
                <input type="text" id="query" name="query" required>
                <button type="submit">Найти</button>
            </form>
        </body>
        </html>
    "#;

    HttpResponse::Ok().content_type("text/html; charset=utf-8").body(html)
}

async fn fetch_news(query: web::Query<QueryParams>) -> impl Responder {
    let client = Client::new();
    let symbol = &query.query.to_uppercase();

    let url = format!("https://min-api.cryptocompare.com/data/v2/news/?categories={}", symbol);

    let response = client.get(&url).send().await;

    match response {
        Ok(resp) => match resp.json::<serde_json::Value>().await {
            Ok(json) => {
                let mut result = format!(
                    r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Новости по {symbol}</title>
</head>
<body>
    <h1>Новости по {symbol}</h1>
    <ul>
"#,
                    symbol = symbol
                );

                if let Some(articles) = json.get("Data").and_then(|d| d.as_array()) {
                    for article in articles.iter().take(10) {
                        let title = article.get("title").and_then(|t| t.as_str()).unwrap_or("Без названия");
                        let link = article.get("url").and_then(|u| u.as_str()).unwrap_or("#");
                        let source = article.get("source").and_then(|s| s.as_str()).unwrap_or("Источник неизвестен");
                        let date = article.get("published_on").and_then(|d| d.as_i64()).unwrap_or(0);
                        let readable_date = NaiveDateTime::from_timestamp_opt(date, 0)
                            .map(|d| d.format("%Y-%m-%d %H:%M").to_string())
                            .unwrap_or("Неизвестная дата".into());

                        result.push_str(&format!(
                            r#"<li><b>{}</b> (<i>{}</i>, {})<br><a href="{}" target="_blank">Читать далее</a></li><br>"#,
                            title, source, readable_date, link
                        ));
                    }
                    result.push_str("</ul></body></html>");
                    HttpResponse::Ok().content_type("text/html; charset=utf-8").body(result)
                } else {
                    HttpResponse::Ok().body("Нет новостей")
                }
            }
            Err(_) => HttpResponse::InternalServerError().body("Ошибка при разборе JSON"),
        },
        Err(_) => HttpResponse::InternalServerError().body("Ошибка запроса к API"),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("🚀 Сервер запущен на http://localhost:8080");
    HttpServer::new(|| {
        App::new()
            .route("/news", web::get().to(fetch_news))
            .service(index)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
