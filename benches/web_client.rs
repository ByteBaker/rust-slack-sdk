//! Benchmarks for Web API client operations

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use slack_rs::web::SlackResponse;
use std::collections::HashMap;

fn bench_slack_response_parsing(c: &mut Criterion) {
    let body =
        r#"{"ok":true,"channel":"C123456","ts":"1234567890.123456","message":{"text":"Hello"}}"#;

    c.bench_function("slack_response_parse", |b| {
        b.iter(|| {
            let response = SlackResponse {
                url: "https://slack.com/api/chat.postMessage".to_string(),
                status_code: 200,
                headers: HashMap::new(),
                body: Some(body.to_string()),
            };
            black_box(response.deserialize_payload::<serde_json::Value>())
        })
    });
}

fn bench_response_error_detection(c: &mut Criterion) {
    let ok_body = r#"{"ok":true,"channel":"C123456"}"#;
    let err_body = r#"{"ok":false,"error":"invalid_auth"}"#;

    let mut group = c.benchmark_group("response_error_detection");

    group.bench_function("ok_response", |b| {
        b.iter(|| {
            let response = SlackResponse {
                url: "https://slack.com/api/test".to_string(),
                status_code: 200,
                headers: HashMap::new(),
                body: Some(ok_body.to_string()),
            };
            black_box(response.is_ok())
        })
    });

    group.bench_function("error_response", |b| {
        b.iter(|| {
            let response = SlackResponse {
                url: "https://slack.com/api/test".to_string(),
                status_code: 200,
                headers: HashMap::new(),
                body: Some(err_body.to_string()),
            };
            black_box(response.is_ok())
        })
    });

    group.finish();
}

fn bench_json_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_serialization");

    group.bench_function("small_payload", |b| {
        let data = serde_json::json!({
            "channel": "C123456",
            "text": "Hello, world!"
        });
        b.iter(|| black_box(serde_json::to_string(&data).unwrap()))
    });

    group.bench_function("medium_payload", |b| {
        let data = serde_json::json!({
            "channel": "C123456",
            "text": "Hello, world!",
            "attachments": [
                {
                    "color": "good",
                    "title": "Title",
                    "text": "Text here",
                    "fields": [
                        {"title": "Field 1", "value": "Value 1", "short": true},
                        {"title": "Field 2", "value": "Value 2", "short": true},
                    ]
                }
            ]
        });
        b.iter(|| black_box(serde_json::to_string(&data).unwrap()))
    });

    group.finish();
}

fn bench_url_building(c: &mut Criterion) {
    c.bench_function("url_with_query_params", |b| {
        b.iter(|| {
            let base = "https://slack.com/api/chat.postMessage";
            let channel = "C123456";
            let text = "Hello, world!";
            black_box(format!("{}?channel={}&text={}", base, channel, text))
        })
    });
}

criterion_group!(
    benches,
    bench_slack_response_parsing,
    bench_response_error_detection,
    bench_json_serialization,
    bench_url_building
);
criterion_main!(benches);
