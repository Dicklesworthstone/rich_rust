//! Thread safety tests for rich_rust.
//!
//! This module verifies:
//! 1. All public types are Send + Sync (compile-time verification)
//! 2. Global caches work correctly under concurrent access
//! 3. Parallel rendering operations are safe

use rich_rust::prelude::*;
use std::thread;

// ============================================================================
// COMPILE-TIME SEND + SYNC VERIFICATION
// ============================================================================

/// Helper function to verify a type is Send + Sync at compile time.
fn assert_send_sync<T: Send + Sync>() {}

#[test]
fn test_color_types_are_send_sync() {
    assert_send_sync::<Color>();
    assert_send_sync::<ColorSystem>();
    assert_send_sync::<ColorType>();
    assert_send_sync::<ColorTriplet>();
}

#[test]
fn test_style_types_are_send_sync() {
    assert_send_sync::<Style>();
    assert_send_sync::<Attributes>();
}

#[test]
fn test_segment_is_send_sync() {
    assert_send_sync::<Segment>();
}

#[test]
fn test_text_types_are_send_sync() {
    assert_send_sync::<Text>();
    assert_send_sync::<Span>();
    assert_send_sync::<JustifyMethod>();
    assert_send_sync::<OverflowMethod>();
}

#[test]
fn test_console_types_are_send_sync() {
    // Console is Send + Sync; output stream is guarded by a Mutex.
    assert_send_sync::<Console>();
    assert_send_sync::<ConsoleOptions>();
}

#[test]
fn test_measurement_is_send_sync() {
    assert_send_sync::<Measurement>();
}

#[test]
fn test_box_chars_is_send_sync() {
    assert_send_sync::<BoxChars>();
}

#[test]
fn test_renderable_types_are_send_sync() {
    assert_send_sync::<Align>();
    assert_send_sync::<AlignMethod>();
    assert_send_sync::<VerticalAlignMethod>();
    assert_send_sync::<Columns>();
    assert_send_sync::<Rule>();
    assert_send_sync::<Panel>();
    assert_send_sync::<Table>();
    assert_send_sync::<Layout>();
    assert_send_sync::<Column>();
    assert_send_sync::<Row>();
    assert_send_sync::<Cell>();
    assert_send_sync::<PaddingDimensions>();
    assert_send_sync::<VerticalAlign>();
    assert_send_sync::<ProgressBar>();
    assert_send_sync::<BarStyle>();
    assert_send_sync::<Spinner>();
    assert_send_sync::<Tree>();
    assert_send_sync::<TreeNode>();
    assert_send_sync::<TreeGuides>();
}

// ============================================================================
// CONCURRENT CACHE ACCESS TESTS
// ============================================================================

#[test]
fn test_concurrent_color_parsing() {
    // Spawn multiple threads that all parse colors concurrently
    let handles: Vec<_> = (0..8)
        .map(|i| {
            thread::spawn(move || {
                for j in 0..500 {
                    // Parse various color formats
                    let _ = Color::parse("red").unwrap();
                    let _ = Color::parse("bright_blue").unwrap();
                    let _ = Color::parse("#ff0000").unwrap();
                    let _ = Color::parse("#abc").unwrap();
                    let _ = Color::parse(&format!("color({})", (i * 50 + j) % 256)).unwrap();
                    let _ = Color::parse("rgb(100, 150, 200)").unwrap();
                    let _ = Color::parse("default").unwrap();
                }
            })
        })
        .collect();

    // All threads should complete without panic
    for handle in handles {
        handle
            .join()
            .expect("Thread panicked during concurrent color parsing");
    }
}

#[test]
fn test_concurrent_style_parsing() {
    // Spawn multiple threads that all parse styles concurrently
    let handles: Vec<_> = (0..8)
        .map(|_| {
            thread::spawn(|| {
                for _ in 0..500 {
                    // Parse various style formats
                    let _ = Style::parse("bold").unwrap();
                    let _ = Style::parse("italic red").unwrap();
                    let _ = Style::parse("bold underline green on white").unwrap();
                    let _ = Style::parse("dim cyan").unwrap();
                    let _ = Style::parse("none").unwrap();
                    let _ = Style::parse("reverse").unwrap();
                }
            })
        })
        .collect();

    // All threads should complete without panic
    for handle in handles {
        handle
            .join()
            .expect("Thread panicked during concurrent style parsing");
    }
}

#[test]
fn test_concurrent_cell_len_calculation() {
    use rich_rust::cells::cell_len;

    // Spawn multiple threads that all calculate cell lengths concurrently
    let handles: Vec<_> = (0..8)
        .map(|i| {
            thread::spawn(move || {
                for _ in 0..500 {
                    // Calculate cell lengths for various strings
                    let _ = cell_len("Hello, World!");
                    let _ = cell_len("Bold text");
                    let _ = cell_len(&format!("Thread {} testing", i));
                    // Wide characters (CJK)
                    let _ = cell_len("\u{4e2d}\u{6587}"); // Chinese characters
                    let _ = cell_len("\u{65e5}\u{672c}\u{8a9e}"); // Japanese
                    // Emoji
                    let _ = cell_len("\u{1f600}\u{1f601}\u{1f602}");
                    // Empty
                    let _ = cell_len("");
                }
            })
        })
        .collect();

    // All threads should complete without panic
    for handle in handles {
        handle
            .join()
            .expect("Thread panicked during concurrent cell_len calculation");
    }
}

// ============================================================================
// CONCURRENT RENDERING TESTS
// ============================================================================

#[test]
fn test_concurrent_text_rendering() {
    let handles: Vec<_> = (0..4)
        .map(|i| {
            thread::spawn(move || {
                for _ in 0..100 {
                    let text = Text::from(format!("Thread {} [bold]testing[/] rendering", i));
                    let _segments = text.render("\n");

                    // Also test with explicit styles
                    let text2 = Text::styled("Styled text", Style::new().bold());
                    let _segments2 = text2.render("\n");
                }
            })
        })
        .collect();

    for handle in handles {
        handle
            .join()
            .expect("Thread panicked during concurrent text rendering");
    }
}

#[test]
fn test_concurrent_table_rendering() {
    let handles: Vec<_> = (0..4)
        .map(|i| {
            thread::spawn(move || {
                for j in 0..50 {
                    let title = format!("Thread {} Table {}", i, j);
                    let row_val = format!("Row {}", j);
                    let mut table = Table::new()
                        .title(title.as_str())
                        .with_column(Column::new("Name"))
                        .with_column(Column::new("Value"));

                    table.add_row_cells([row_val.as_str(), "Data"]);
                    table.add_row_cells(["Test", "123"]);

                    let _segments = table.render(80);
                }
            })
        })
        .collect();

    for handle in handles {
        handle
            .join()
            .expect("Thread panicked during concurrent table rendering");
    }
}

#[test]
fn test_concurrent_panel_rendering() {
    let handles: Vec<_> = (0..4)
        .map(|i| {
            thread::spawn(move || {
                for j in 0..100 {
                    let content = format!("Thread {} Panel {}", i, j);
                    let panel = Panel::from_text(content.as_str())
                        .title("Test Panel")
                        .width(40);

                    let _segments = panel.render(80);
                }
            })
        })
        .collect();

    for handle in handles {
        handle
            .join()
            .expect("Thread panicked during concurrent panel rendering");
    }
}

#[test]
fn test_concurrent_progress_bar_rendering() {
    let handles: Vec<_> = (0..4)
        .map(|_| {
            thread::spawn(|| {
                for completed in 0..=100 {
                    let mut bar = ProgressBar::new().width(40);
                    bar.set_progress(f64::from(completed) / 100.0);

                    let _segments = bar.render(80);
                }
            })
        })
        .collect();

    for handle in handles {
        handle
            .join()
            .expect("Thread panicked during concurrent progress bar rendering");
    }
}

#[test]
fn test_concurrent_rule_rendering() {
    let handles: Vec<_> = (0..4)
        .map(|i| {
            thread::spawn(move || {
                for j in 0..100 {
                    let title = format!("Thread {} Rule {}", i, j);
                    let rule = Rule::with_title(title.as_str());
                    let _segments = rule.render(80);

                    let rule_plain = Rule::new();
                    let _segments2 = rule_plain.render(80);
                }
            })
        })
        .collect();

    for handle in handles {
        handle
            .join()
            .expect("Thread panicked during concurrent rule rendering");
    }
}

#[test]
fn test_concurrent_tree_rendering() {
    let handles: Vec<_> = (0..4)
        .map(|i| {
            thread::spawn(move || {
                for j in 0..50 {
                    let root = TreeNode::new(format!("Root {} {}", i, j))
                        .child(TreeNode::new("Child 1"))
                        .child(TreeNode::new("Child 2"));

                    let tree = Tree::new(root);
                    let _segments = tree.render();
                }
            })
        })
        .collect();

    for handle in handles {
        handle
            .join()
            .expect("Thread panicked during concurrent tree rendering");
    }
}

// ============================================================================
// MIXED CONCURRENT OPERATIONS
// ============================================================================

#[test]
fn test_mixed_concurrent_operations() {
    // This test exercises multiple subsystems concurrently to detect any
    // cross-subsystem thread safety issues

    let handles: Vec<_> = (0..12)
        .map(|i| {
            thread::spawn(move || {
                match i % 6 {
                    0 => {
                        // Color parsing
                        for _ in 0..200 {
                            let _ = Color::parse("red").unwrap();
                            let _ = Color::parse("#ff0000").unwrap();
                        }
                    }
                    1 => {
                        // Style parsing
                        for _ in 0..200 {
                            let _ = Style::parse("bold red").unwrap();
                        }
                    }
                    2 => {
                        // Text rendering
                        for _ in 0..100 {
                            let text = Text::from("[bold]Hello[/]");
                            let _ = text.render("\n");
                        }
                    }
                    3 => {
                        // Table rendering
                        for _ in 0..50 {
                            let mut table = Table::new().with_column(Column::new("A"));
                            table.add_row_cells(["value"]);
                            let _ = table.render(80);
                        }
                    }
                    4 => {
                        // Panel rendering
                        for _ in 0..100 {
                            let panel = Panel::from_text("content");
                            let _ = panel.render(80);
                        }
                    }
                    5 => {
                        // Cell length calculation
                        use rich_rust::cells::cell_len;
                        for _ in 0..200 {
                            let _ = cell_len("test string");
                        }
                    }
                    _ => unreachable!(),
                }
            })
        })
        .collect();

    for handle in handles {
        handle
            .join()
            .expect("Thread panicked during mixed concurrent operations");
    }
}

// ============================================================================
// CACHE CONSISTENCY TESTS
// ============================================================================

#[test]
fn test_color_cache_consistency() {
    // Verify that concurrent cache access returns consistent results
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    let failed = Arc::new(AtomicBool::new(false));

    let handles: Vec<_> = (0..8)
        .map(|_| {
            let failed = Arc::clone(&failed);
            thread::spawn(move || {
                for _ in 0..1000 {
                    let color1 = Color::parse("bright_red").unwrap();
                    let color2 = Color::parse("bright_red").unwrap();

                    // Both should return the same color number
                    if color1.number != color2.number {
                        failed.store(true, Ordering::SeqCst);
                        return;
                    }
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    assert!(
        !failed.load(Ordering::SeqCst),
        "Cache returned inconsistent results"
    );
}

#[test]
fn test_style_cache_consistency() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    let failed = Arc::new(AtomicBool::new(false));

    let handles: Vec<_> = (0..8)
        .map(|_| {
            let failed = Arc::clone(&failed);
            thread::spawn(move || {
                for _ in 0..1000 {
                    let style1 = Style::parse("bold italic red").unwrap();
                    let style2 = Style::parse("bold italic red").unwrap();

                    // Both should return equivalent styles
                    if style1 != style2 {
                        failed.store(true, Ordering::SeqCst);
                        return;
                    }
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    assert!(
        !failed.load(Ordering::SeqCst),
        "Style cache returned inconsistent results"
    );
}
