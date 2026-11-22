// Deterministic data belongs to the isolated verification harness, never to the
// application bundle or its first-launch document.
export function readingFixture(template, now = Math.floor(Date.now() / 1000)) {
  const state = structuredClone(template);
  const sources = [
    ["The Rust Blog", "Technology"], ["NASA", "Science"],
    ["Smashing Magazine", "Design"], ["Mozilla Hacks", "Technology"],
    ["A List Apart", "Design"], ["Hacker News", "Technology"],
  ];
  state.feeds = sources.map(([title, folder], index) => ({
    id: index + 1, title, folder, folders: [folder],
    url: `https://ui-fixture.example/feeds/${index}.xml`,
    site_url: `https://ui-fixture.example/sites/${index}/`,
    description: "Isolated UI verification fixture", fetched_at: now,
    error: null, sample: false,
  }));
  state.articles = [1, 2, 3, 4, 5, 6, 1, 3, 2].map((feed_id, index) => ({
    id: index + 7, feed_id, guid: `ui-verification-${index}`,
    title: `Reading verification item ${index + 1}`,
    url: `https://ui-fixture.example/articles/${index}`,
    author: "UI verification fixture", content_base: "",
    content: "<p>A deterministic article used by the isolated UI verification harness.</p>" +
      "<p>Read, star, annotate and organize this item through native Reader controls.</p>",
    summary: "A deterministic article used by the isolated UI verification harness.",
    published: now - (index * 2100 + 600),
    starred: index === 2, read: index >= 7, read_at: index >= 7 ? now - 3600 : null,
    tags: index === 2 ? ["Inspiration"] : [], note: "",
  }));
  state.folders = ["Design", "Science", "Technology"];
  state.settings = {...template.settings, start_page: "Home"};
  state.stream_settings = {};
  state.next_id = 16;
  return state;
}
