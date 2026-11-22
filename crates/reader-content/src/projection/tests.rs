use super::*;

fn text(block: &Block) -> String {
    block.spans.iter().map(|span| span.text.as_str()).collect()
}

#[test]
fn whitespace_survives_inline_boundaries_without_joining_words() {
    let blocks = parse(
        "<p>Hello <strong>world</strong> <em>and</em>\n <a href='/more'>read more</a>.</p>",
        "https://example.org/feed",
    );
    assert_eq!(blocks.len(), 1);
    assert_eq!(text(&blocks[0]), "Hello world and read more.");
    assert!(blocks[0]
        .spans
        .iter()
        .any(|s| s.bold && s.text.ends_with("world")));
    assert!(blocks[0]
        .spans
        .iter()
        .any(|s| s.italic && s.text.ends_with("and")));
    assert!(blocks[0]
        .spans
        .iter()
        .any(|s| s.href.as_deref() == Some("https://example.org/more")));
}

#[test]
fn preserves_code_indentation_literals_and_entities() {
    let blocks = parse("<pre><code>if a &lt; b {\n    println!(&quot;hi&quot;);\n}</code></pre><p>2 < 3 &amp; 5 > 4 &unknown;</p>", "https://example.org");
    assert!(blocks[0].pre);
    assert_eq!(text(&blocks[0]), "if a < b {\n    println!(\"hi\");\n}");
    assert_eq!(text(&blocks[1]), "2 < 3 & 5 > 4 &unknown;");
}

#[test]
fn complete_named_entities_and_bare_ampersands_are_preserved() {
    let blocks = parse(
        "<p>Caf&eacute; &alpha; &NotEqualTilde; &unknown; &bare &amp; &#128;</p>",
        "https://example.org",
    );
    assert_eq!(text(&blocks[0]), "Café α ≂̸ &unknown; &bare & €");
    let bare = "&".repeat(100_000);
    assert_eq!(text(&parse(&bare, "https://example.org")[0]), bare);
}

#[test]
fn comments_cdata_and_malformed_trailing_text_do_not_drop_readable_body() {
    let blocks = parse(
        "<p>Before<!-- <script> > ignored --> after</p><![CDATA[plain <value>]]><p>broken <strong",
        "https://example.org",
    );
    assert_eq!(text(&blocks[0]), "Before after");
    assert_eq!(text(&blocks[1]), "plain <value>");
    assert_eq!(text(&blocks[2]), "broken <strong");
}

#[test]
fn nested_lists_quotes_and_headings_retain_structure() {
    let blocks = parse("<h2>A <em>heading</em></h2><blockquote><p>Quoted</p><ul><li>Outer<ul><li>Inner</li></ul></li></ul></blockquote><ol start='3'><li>Three</li><li value='8'>Eight</li></ol>", "https://example.org");
    assert_eq!(blocks[0].heading, 2);
    assert!(blocks[1].quote);
    assert!(blocks[2].quote);
    assert_eq!(blocks[2].list_depth, 1);
    assert_eq!(blocks[3].list_depth, 2);
    assert_eq!(text(&blocks[4]), "3.  Three");
    assert_eq!(text(&blocks[5]), "8.  Eight");
}

#[test]
fn tables_preserve_cell_boundaries_spans_and_formatting() {
    let blocks = parse("<table><tr><th colspan='2'>Heading</th></tr><tr><td rowspan='2'><b>A</b></td><td><p>B</p><p>C</p></td></tr><tr><td>D</td></tr></table><p>After</p>", "https://example.org");
    assert_eq!(blocks.len(), 2);
    let table = blocks[0].table.as_ref().unwrap();
    assert_eq!(table.rows.len(), 3);
    assert!(table.rows[0][0].heading);
    assert_eq!(table.rows[0][0].colspan, 2);
    assert_eq!(table.rows[1][0].rowspan, 2);
    assert!(table.rows[1][0].spans[0].bold);
    assert_eq!(
        table.rows[1][1]
            .spans
            .iter()
            .map(|s| s.text.as_str())
            .collect::<String>(),
        "B\nC"
    );
    assert_eq!(text(&blocks[1]), "After");
}

#[test]
fn active_content_and_unsafe_urls_never_become_native_actions() {
    let blocks = parse("<script>if(a<b) bad()</script><style>bad</style><form>private</form><div hidden>hidden<div>nested</div>still hidden</div><p><a href='javascript:bad()'>Visible</a></p><img src='data:x'><iframe src='javascript:bad()'>bad</iframe><img hidden src='/invisible'><p>After</p>", "https://example.org");
    assert_eq!(blocks.len(), 2);
    assert_eq!(text(&blocks[0]), "Visible");
    assert!(blocks[0].spans.iter().all(|s| s.href.is_none()));
    assert_eq!(text(&blocks[1]), "After");
}

#[test]
fn image_metadata_and_media_fallbacks_preserve_original_targets() {
    let blocks = parse("<a href='/article'><img src='/photo.jpg?x=1&amp;y=2' alt='Photo &copy;' width='1200' height='800'></a><img data-src='/lazy.png'><video><source src='/film.mp4'></video><iframe src='https://video.example/embed/1'></iframe>", "https://example.org/feed");
    let image = blocks[0].image.as_ref().unwrap();
    assert_eq!(image.url, "https://example.org/photo.jpg?x=1&y=2");
    assert_eq!(image.alt, "Photo ©");
    assert_eq!(image.width, Some(1200.0));
    assert_eq!(image.height, Some(800.0));
    assert_eq!(image.link.as_deref(), Some("https://example.org/article"));
    assert_eq!(
        blocks[1].image.as_ref().unwrap().url,
        "https://example.org/lazy.png"
    );
    assert_eq!(
        blocks[2].spans[0].href.as_deref(),
        Some("https://example.org/film.mp4")
    );
    assert_eq!(
        blocks[3].spans[0].href.as_deref(),
        Some("https://video.example/embed/1")
    );
}

#[test]
fn lazy_and_responsive_images_choose_real_safe_sources() {
    let blocks = parse("<img src='data:image/gif;base64,ignored' data-src='/real.jpg'><img srcset='/small.jpg 320w, /retina.jpg 1300w, /large.jpg 4000w'><img srcset='data:image/png;base64,not-a-url 1x'>", "https://example.org");
    assert_eq!(blocks.len(), 2);
    assert_eq!(
        blocks[0].image.as_ref().unwrap().url,
        "https://example.org/real.jpg"
    );
    assert_eq!(
        blocks[1].image.as_ref().unwrap().url,
        "https://example.org/retina.jpg"
    );
}

#[test]
fn resource_limits_keep_a_readable_prefix_and_report_the_remainder() {
    let source = "<p>Paragraph</p>".repeat(MAX_BLOCKS + 10);
    let blocks = parse(&source, "https://example.org");
    assert_eq!(blocks.len(), MAX_BLOCKS + 1);
    assert_eq!(text(&blocks[0]), "Paragraph");
    assert!(blocks.last().unwrap().notice);
    let huge = "word ".repeat(MAX_INPUT / 5 + 1);
    let blocks = parse(&huge, "https://example.org");
    assert!(!text(&blocks[0]).is_empty());
    assert!(blocks.last().unwrap().notice);
}
