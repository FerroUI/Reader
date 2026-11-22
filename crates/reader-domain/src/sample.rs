use crate::{AppState, Article, Feed};

impl AppState {
    /// Deterministic test data. This module is excluded from application builds.
    pub fn sample(now: i64) -> Self {
        let mut state = Self::new();
        let feeds = [
            (
                "The Rust Blog",
                "https://blog.rust-lang.org/feed.xml",
                "https://blog.rust-lang.org/",
                "Technology",
            ),
            (
                "NASA",
                "https://www.nasa.gov/feed/",
                "https://www.nasa.gov/",
                "Science",
            ),
            (
                "Smashing Magazine",
                "https://www.smashingmagazine.com/feed/",
                "https://www.smashingmagazine.com/",
                "Design",
            ),
            (
                "Mozilla Hacks",
                "https://hacks.mozilla.org/feed/",
                "https://hacks.mozilla.org/",
                "Technology",
            ),
            (
                "A List Apart",
                "https://alistapart.com/main/feed/",
                "https://alistapart.com/",
                "Design",
            ),
            (
                "Hacker News",
                "https://hnrss.org/frontpage",
                "https://news.ycombinator.com/",
                "Technology",
            ),
        ];
        for (title, url, site_url, folder) in feeds {
            let id = state.allocate();
            state.feeds.push(Feed { id, title: title.into(), url: url.into(), site_url: site_url.into(),
                folder: folder.into(), folders: vec![folder.into()], description: "Starter subscription with illustrative sample articles. Refresh to fetch the publisher's live feed.".into(), sample: true, ..Feed::default() });
        }
        let stories = [
            (0, "A familiar place for everything you want to read", "Your reading list, beautifully simple", "Welcome back to a quieter corner of the web. Reader brings your favorite sites together in one place, with the familiar subscriptions, folders, stars, and keyboard shortcuts you remember.", "Subscribe to a feed, or import your old OPML file in Reader settings. This starter library contains original sample articles to help you explore the interface. Refresh to load the publishers' actual articles."),
            (1, "A little curiosity goes a very long way", "Discoveries worth slowing down for", "There is something wonderful about opening your reading list and finding an idea you did not know you were looking for. Follow the science, save the stories, and keep a few good questions close at hand.", "This is an original sample article, not a NASA publication. Choose Refresh to replace samples with the latest available feed entries."),
            (2, "Designing for the way people actually read", "The details that make reading feel effortless", "Good reading experiences begin with a few quiet choices: a comfortable line length, clear hierarchy, generous space around the text, and controls that stay where you expect them.", "Try the expanded view for full articles, or switch to list view to scan headlines. Press j and k to move through entries, s to star a story, and m to mark it unread."),
            (3, "The open web is still full of good things", "Small sites. Big ideas. One reading list.", "Feeds are a simple promise: a site publishes something new, and your reader brings it to you. No algorithmic timeline, no pressure to keep scrolling, and no need to remember which tabs you left open.", "This illustrative story is part of the starter library. Add your own subscriptions with the red Subscribe button, and organize them into whatever folders make sense to you."),
            (4, "Making room for a better reading habit", "A few favorites, collected in one place", "A great reading list is personal. Start with a handful of sites you trust, add a few that surprise you, and give yourself permission to mark the rest as read.", "Stars keep your favorite articles close at hand. Tags let one story belong to several collections. Everything is saved in this browser, and you can export a backup from Reader settings."),
            (5, "What are you building this weekend?", "Ideas from a community of curious people", "A new project often starts with an interesting question, a useful tool, or a conversation that sends you down an unexpected path. Keep the things that spark your curiosity together.", "Search your library by title, text, author, tag, or subscription. Try is:unread, is:starred, from:rust, or a phrase in quotation marks. This is an original sample, not a Hacker News item."),
            (0, "Small tools, thoughtfully put together", "Reliable software starts with clear boundaries", "A reader does a few essential things: fetch a feed, remember what you have read, and get out of the way. Keeping those responsibilities separate makes a small application easier to understand and faster to use.", "This original demonstration article introduces the reading controls. Open the article menu to add tags, email a link, or open the publisher's site."),
            (2, "The beauty of a well-organized sidebar", "A place for every subscription", "Folders give a growing list of subscriptions some welcome structure. Group feeds by subject, by project, or simply by the time you like to read them.", "Open Reader settings to rename or move subscriptions. Import and export use the standard OPML format, so your reading list remains portable."),
            (1, "A new perspective on the everyday", "Look closer, and keep asking questions", "The best discoveries often begin with patient observation. A familiar landscape, a simple measurement, or a question asked in a different way can reveal something entirely new.", "This is an original sample article included with Reader. Use the refresh control to fetch real articles from your subscriptions."),
        ];
        for (index, (feed, title, subtitle, first, second)) in stories.into_iter().enumerate() {
            let id = state.allocate();
            let source = &state.feeds[feed];
            let content = format!("<p><strong>{subtitle}</strong></p><p>{first}</p><p>{second}</p><p><em>Illustrative sample · FerroUI Reader</em></p>");
            let mut article = Article {
                id,
                feed_id: source.id,
                guid: format!("reader-sample-{index}"),
                title: title.into(),
                url: source.site_url.clone(),
                author: "Reader sample library".into(),
                summary: first.into(),
                content,
                published: now - (index as i64 * 2100 + 600),
                starred: index == 2,
                read: index >= 7,
                read_at: (index >= 7).then_some(now - 3600),
                tags: if index == 2 {
                    vec!["Inspiration".into()]
                } else {
                    vec![]
                },
                ..Article::default()
            };
            article.rebuild_search();
            state.articles.push(article);
        }
        state.folders = vec!["Technology".into(), "Science".into(), "Design".into()];
        state.folders.sort();
        state.reindex();
        state
    }
}
