import assert from "node:assert/strict";
import { fixture, BASE, replace, navigate } from "./ordering-fixture.mjs";

export async function japaneseChecks(h, check) {
  const baseline = await h.state(), viewport = await h.snapshot();
  const route = await h.page.evaluate("readerApp.currentView()");
  try {
    await check("bundled Japanese fallback covers native headings, rich text and code without replacing Latin faces", async () => {
      const data = fixture(baseline);
      const article = data.articles.find(article => article.feed_id === BASE + 3);
      article.title = "日本語のニュース — Reader";
      article.content = "<p>日本語の記事を読む。ひらがな、カタカナ、漢字。「読書」を楽しむ。</p>"
        + "<p>Latin text keeps its original font. <strong>太字の日本語</strong> and <em>斜体の日本語</em>.</p>"
        + "<pre><code>let 挨拶 = \"こんにちは、世界。\";\n// 日本語と ASCII code</code></pre>"
        + "<blockquote>読む・考える・共有する。</blockquote>";
      await h.resize(1363, 936); await replace(h, data); await navigate(h, {Feed: BASE + 3});
      const result = await h.page.evaluate(`(() => {
        const fonts = ['Liberation Sans', 'Liberation Serif', 'Liberation Mono'];
        return fonts.flatMap(family => [[false,false],[true,false],[false,true],[true,true]].map(([bold,italic]) => ({
          family, bold, italic, glyphs: JSON.parse(readerApp.inspectGlyphs('AiW日本語かなカナ、。「」', 'fonts:Reader#' + family, bold, italic))
        })));
      })()`);
      for (const sample of result) for (const glyph of sample.glyphs) {
        assert(glyph.glyph > 0 && glyph.advance > 0, JSON.stringify({sample, glyph}));
        assert.equal(glyph.family, /^[AiW]$/.test(glyph.character) ? sample.family : "Noto Sans JP");
      }
      const mono = result.find(sample => sample.family === "Liberation Mono" && !sample.bold && !sample.italic).glyphs;
      assert.equal(mono.find(glyph => glyph.character === "i").advance, mono.find(glyph => glyph.character === "W").advance);
      await h.control(e => e.name === "ArticleHeadlineText" && e.text === article.title, "Japanese native headline");
      await h.capture("24-japanese-desktop");
      await h.resize(320, 844);
      await h.control(e => e.name === "ArticleHeadlineText" && e.text === article.title, "Japanese phone headline");
      await h.capture("24-japanese-phone");
      await h.record("24-japanese-glyphs", result);
    });
  } catch (error) {
    await h.capture("24-japanese-failure").catch(() => {}); throw error;
  } finally {
    await replace(h, baseline); await h.page.evaluate(`readerApp.navigate(${JSON.stringify(route)})`);
    await h.resize(viewport.width, viewport.height);
  }
}
