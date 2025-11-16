//! Benchmarks for Block Kit serialization/deserialization

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use slack_rs::models::*;

fn bench_section_block_serialize(c: &mut Criterion) {
    let block = SectionBlock::new("This is a test section").unwrap();

    c.bench_function("section_block_serialize", |b| {
        b.iter(|| black_box(serde_json::to_string(&block).unwrap()))
    });
}

fn bench_section_block_deserialize(c: &mut Criterion) {
    let json = r#"{"type":"section","text":{"type":"mrkdwn","text":"This is a test section"}}"#;

    c.bench_function("section_block_deserialize", |b| {
        b.iter(|| black_box(serde_json::from_str::<SectionBlock>(json).unwrap()))
    });
}

fn bench_complex_message_serialize(c: &mut Criterion) {
    let blocks: Vec<Block> = vec![
        HeaderBlock::new("Test Header").unwrap().into(),
        DividerBlock::new().into(),
        SectionBlock::new("*Bold text* and _italic text_")
            .unwrap()
            .into(),
        ActionsBlock::builder()
            .elements(vec![
                ButtonElement::new("Click me", "button_1")
                    .with_style(ButtonStyle::Primary)
                    .build()
                    .unwrap(),
                ButtonElement::new("Cancel", "button_2")
                    .with_style(ButtonStyle::Danger)
                    .build()
                    .unwrap(),
            ])
            .build()
            .unwrap()
            .into(),
    ];

    c.bench_function("complex_message_serialize", |b| {
        b.iter(|| black_box(serde_json::to_string(&blocks).unwrap()))
    });
}

fn bench_modal_view_serialize(c: &mut Criterion) {
    let view = View::modal()
        .title("Test Modal")
        .submit("Submit")
        .close("Cancel")
        .blocks(vec![
            InputBlock::new("Name")
                .element(PlainTextInputElement::new("name_input"))
                .build()
                .unwrap(),
            InputBlock::new("Email")
                .element(PlainTextInputElement::new("email_input"))
                .build()
                .unwrap(),
        ])
        .build()
        .unwrap();

    c.bench_function("modal_view_serialize", |b| {
        b.iter(|| black_box(serde_json::to_string(&view).unwrap()))
    });
}

fn bench_button_element_creation(c: &mut Criterion) {
    c.bench_function("button_element_creation", |b| {
        b.iter(|| {
            black_box(
                ButtonElement::new("Test Button", "test_action")
                    .with_style(ButtonStyle::Primary)
                    .with_url("https://example.com")
                    .build()
                    .unwrap(),
            )
        })
    });
}

criterion_group!(
    benches,
    bench_section_block_serialize,
    bench_section_block_deserialize,
    bench_complex_message_serialize,
    bench_modal_view_serialize,
    bench_button_element_creation
);
criterion_main!(benches);
