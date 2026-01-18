//! Benchmarks for rich_rust rendering.

use criterion::{Criterion, criterion_group, criterion_main};
use rich_rust::cells::cell_len;
use rich_rust::color::Color;
use rich_rust::prelude::*;
use rich_rust::style::Style;
use rich_rust::text::Text;
use std::hint::black_box;

fn benchmark_text_render(c: &mut Criterion) {
    let mut text = Text::new("Hello, World! This is a test string for benchmarking.");
    text.stylize(0, 5, Style::new().bold());
    text.stylize(7, 12, Style::new().italic());

    c.bench_function("text_render", |b| {
        b.iter(|| black_box(text.render("")));
    });
}

fn benchmark_text_wrap(c: &mut Criterion) {
    let text = Text::new(
        "This is a longer string that needs to be wrapped to fit within a certain width. It contains multiple words and should demonstrate the wrapping algorithm.",
    );

    c.bench_function("text_wrap_80", |b| {
        b.iter(|| black_box(text.wrap(80)));
    });

    c.bench_function("text_wrap_40", |b| {
        b.iter(|| black_box(text.wrap(40)));
    });
}

fn benchmark_style_parse(c: &mut Criterion) {
    c.bench_function("style_parse_simple", |b| {
        b.iter(|| black_box(Style::parse("bold red")));
    });

    c.bench_function("style_parse_complex", |b| {
        b.iter(|| black_box(Style::parse("bold italic underline red on blue")));
    });
}

fn benchmark_style_render(c: &mut Criterion) {
    use rich_rust::color::ColorSystem;

    let simple_style = Style::new().bold();
    let complex_style = Style::new()
        .bold()
        .italic()
        .color(Color::from_rgb(255, 100, 50))
        .bgcolor(Color::from_rgb(0, 50, 100));
    let link_style = Style::new()
        .bold()
        .link("https://example.com/page");
    let text = "Hello, World!";

    c.bench_function("style_render_simple", |b| {
        b.iter(|| black_box(simple_style.render(text, ColorSystem::TrueColor)));
    });

    c.bench_function("style_render_complex", |b| {
        b.iter(|| black_box(complex_style.render(text, ColorSystem::TrueColor)));
    });

    c.bench_function("style_render_with_link", |b| {
        b.iter(|| black_box(link_style.render(text, ColorSystem::TrueColor)));
    });

    c.bench_function("style_make_ansi_codes", |b| {
        b.iter(|| black_box(complex_style.make_ansi_codes(ColorSystem::TrueColor)));
    });

    // Test buffer reuse pattern
    c.bench_function("style_make_ansi_codes_into", |b| {
        let mut buffer = String::with_capacity(64);
        b.iter(|| {
            buffer.clear();
            complex_style.make_ansi_codes_into(ColorSystem::TrueColor, &mut buffer);
            black_box(buffer.len())
        });
    });
}

fn benchmark_color_parse(c: &mut Criterion) {
    c.bench_function("color_parse_named", |b| {
        b.iter(|| black_box(Color::parse("red")));
    });

    c.bench_function("color_parse_hex", |b| {
        b.iter(|| black_box(Color::parse("#ff5733")));
    });

    c.bench_function("color_parse_rgb", |b| {
        b.iter(|| black_box(Color::parse("rgb(255, 87, 51)")));
    });

    c.bench_function("color_parse_indexed", |b| {
        b.iter(|| black_box(Color::parse("color(196)")));
    });
}

fn benchmark_cell_len(c: &mut Criterion) {
    let ascii = "Hello, World!";
    let cjk = "你好世界こんにちは";
    let emoji = "Hello 👋🌍🎉 World";
    let mixed = "Hello 你好 👋 World こんにちは";
    let long_ascii = "a".repeat(100);

    c.bench_function("cell_len_ascii_short", |b| {
        b.iter(|| black_box(cell_len(ascii)));
    });

    c.bench_function("cell_len_cjk", |b| {
        b.iter(|| black_box(cell_len(cjk)));
    });

    c.bench_function("cell_len_emoji", |b| {
        b.iter(|| black_box(cell_len(emoji)));
    });

    c.bench_function("cell_len_mixed", |b| {
        b.iter(|| black_box(cell_len(mixed)));
    });

    c.bench_function("cell_len_long_ascii", |b| {
        b.iter(|| black_box(cell_len(&long_ascii)));
    });
}

fn benchmark_table_render(c: &mut Criterion) {
    // Small table: 3x3
    let mut small_table = Table::new();
    small_table = small_table
        .with_column(Column::new("A"))
        .with_column(Column::new("B"))
        .with_column(Column::new("C"));
    small_table.add_row_cells(["1", "2", "3"]);
    small_table.add_row_cells(["4", "5", "6"]);
    small_table.add_row_cells(["7", "8", "9"]);

    c.bench_function("table_render_3x3", |b| {
        b.iter(|| {
            let segments: Vec<_> = black_box(small_table.render(80));
            black_box(segments)
        });
    });

    // Medium table: 10x5
    let mut medium_table = Table::new();
    medium_table = medium_table
        .with_column(Column::new("Name"))
        .with_column(Column::new("Age"))
        .with_column(Column::new("City"))
        .with_column(Column::new("Country"))
        .with_column(Column::new("Score"));
    for i in 0..10 {
        medium_table.add_row_cells([
            format!("User{i}"),
            format!("{}", 20 + i),
            "New York".to_string(),
            "USA".to_string(),
            format!("{}", 80 + i),
        ]);
    }

    c.bench_function("table_render_10x5", |b| {
        b.iter(|| {
            let segments: Vec<_> = black_box(medium_table.render(120));
            black_box(segments)
        });
    });
}

fn benchmark_panel_render(c: &mut Criterion) {
    let panel = Panel::from_text("This is a panel with some content inside.")
        .title("Title")
        .subtitle("Subtitle")
        .width(60);

    c.bench_function("panel_render", |b| {
        b.iter(|| {
            let segments: Vec<_> = black_box(panel.render(80));
            black_box(segments)
        });
    });
}

criterion_group!(
    benches,
    benchmark_text_render,
    benchmark_text_wrap,
    benchmark_style_parse,
    benchmark_style_render,
    benchmark_color_parse,
    benchmark_cell_len,
    benchmark_table_render,
    benchmark_panel_render,
);
criterion_main!(benches);
