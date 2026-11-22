use super::*;
#[test]
fn projects_safe_content_and_formatting() {
    let blocks=parse("<p>Hello <strong>world</strong> <a href='/more'>read more</a></p><script>bad()</script><img src='/photo.jpg' alt='Photo'>","https://example.org/feed");
    assert_eq!(blocks.len(), 2);
    assert!(blocks[0].spans.iter().any(|s| s.bold && s.text == "world"));
    assert!(blocks[0]
        .spans
        .iter()
        .any(|s| s.href.as_deref() == Some("https://example.org/more")));
    assert_eq!(
        blocks[1].image.as_ref().unwrap().0,
        "https://example.org/photo.jpg"
    );
}
#[test]
fn preserves_code_lists_and_table_cells() {
    let blocks=parse("<pre><code>fn main() {\n    println!(&quot;hi&quot;);\n}</code></pre><ol start='3'><li>Three</li><li>Four</li></ol><table><tr><th>Key</th><th>Value</th></tr><tr><td>A</td><td>B</td></tr></table>","https://example.org");
    assert!(blocks[0].pre);
    assert!(blocks[0].spans[0]
        .text
        .contains("\n    println!(\"hi\");\n"));
    assert_eq!(blocks[1].spans[0].text, "3.  ");
    assert_eq!(blocks[2].spans[0].text, "4.  ");
    assert!(blocks[3].spans.iter().any(|span| span.text == "    "));
    assert_eq!(blocks.len(), 5);
}
#[test]
fn discards_active_urls() {
    let blocks = parse(
        "<p><a href='javascript:bad()'>safe text</a></p><img src='data:x'>",
        "https://example.org",
    );
    assert_eq!(blocks.len(), 1);
    assert!(blocks[0].spans.iter().all(|s| s.href.is_none()));
}
