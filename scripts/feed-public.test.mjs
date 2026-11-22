import test from "node:test";
import assert from "node:assert/strict";
import { PUBLIC_FEED_SERVICE, PUBLIC_FEED_ENDPOINT, normalizePublicFeed } from "../crates/reader-browser/wwwroot/feed-public.js";

const publisher = "https://publisher.example/blog/feed.xml?edition=world";
const normalize = (items = [], feed = {}) => JSON.parse(normalizePublicFeed(JSON.stringify({status:"ok",feed,items}), publisher));

test("public feed normalization retains publisher identity, full content and RSS metadata", () => {
  assert.equal(PUBLIC_FEED_SERVICE, "rss2json");
  assert.equal(PUBLIC_FEED_ENDPOINT, "https://api.rss2json.com/v1/api.json?rss_url={url}");
  const feed = normalize([{
    guid:"opaque:article:123",title:"An article",link:"../stories/123#chapter",pubDate:"2026-11-22 10:20:30",
    content:"<p>Full <strong>article</strong> content.</p>",description:"Short description",author:"Ada",
    thumbnail:"./cover.jpg",categories:["Science","Rust"],
    enclosure:{link:"https://media.example/episode.mp3",type:"audio/mpeg",length:"2048",duration:"42.5",title:"Episode"},
  }], {url:"https://api.rss2json.com/internal/feed",title:"Publisher",link:"../",description:"Feed description",image:"/icon.png",author:"Editorial"});
  assert.deepEqual(feed, {
    version:"https://jsonfeed.org/version/1.1",feed_url:publisher,title:"Publisher",
    home_page_url:"https://publisher.example/",description:"Feed description",icon:"https://publisher.example/icon.png",authors:[{name:"Editorial"}],
    items:[{id:"opaque:article:123",title:"An article",url:"https://publisher.example/stories/123#chapter",
      content_html:"<p>Full <strong>article</strong> content.</p>",summary:"Short description",authors:[{name:"Ada"}],date_published:"2026-11-22T10:20:30.000Z",
      image:"https://publisher.example/stories/cover.jpg",tags:["Science","Rust"],
      attachments:[{url:"https://media.example/episode.mp3",mime_type:"audio/mpeg",title:"Episode",size_in_bytes:2048,duration_in_seconds:42.5}]}],
  });
});

test("Atom-like identifiers and explicit date offsets survive service normalization", () => {
  const feed = normalize([
    {guid:"tag:publisher.example,2026:post/42",content:"<p>Atom body</p>",pubDate:"2026-11-22T10:20:30+02:00"},
    {guid:"urn:uuid:0b6c67c0-992f-4c8e-bc19-5b968268870d",pubDate:"2026-11-22"},
    {guid:"rfc-date",pubDate:"Sun, 22 Nov 2026 10:20:30 GMT"},
  ]);
  assert.equal(feed.items[0].id, "tag:publisher.example,2026:post/42");
  assert.equal(feed.items[0].date_published, "2026-11-22T08:20:30.000Z");
  assert.equal(feed.items[1].id, "urn:uuid:0b6c67c0-992f-4c8e-bc19-5b968268870d");
  assert.equal(feed.items[1].date_published, "2026-11-22T00:00:00.000Z");
  assert.equal(feed.items[2].date_published, "2026-11-22T10:20:30.000Z");
});

test("missing strings use description or empty content without inventing authors or dates", () => {
  const feed = normalize([
    {guid:null,title:4,link:"/story",content:"",description:"<p>Description body</p>",author:{name:"wrong shape"},pubDate:"invalid",categories:[null,42,"Tag"]},
    {guid:"empty",content:null,description:false,pubDate:"2026-02-30 12:00:00",enclosure:[]},
    {pubDate:"2026-11-22 25:00:00"},
    {pubDate:"42"},
    {pubDate:"3/4/26"},
  ]);
  assert.deepEqual(feed.items[0], {id:"https://publisher.example/story",url:"https://publisher.example/story",content_html:"<p>Description body</p>",summary:"<p>Description body</p>",tags:["Tag"]});
  assert.deepEqual(feed.items[1], {id:"empty",content_html:""});
  assert.deepEqual(feed.items[2], {content_html:""});
  assert.deepEqual(feed.items[3], {content_html:""});
  assert.deepEqual(feed.items[4], {content_html:""});
  assert.equal(feed.title, publisher);
  assert.equal("authors" in feed, false);
});

test("an empty publisher feed is valid and provider HTTP-like errors remain explicit", () => {
  assert.deepEqual(normalize([], {title:"Empty"}), {version:"https://jsonfeed.org/version/1.1",feed_url:publisher,title:"Empty",items:[]});
  assert.throws(() => normalizePublicFeed('{"status":"error","message":"Feed was not found"}', publisher), /rss2json could not load this feed: Feed was not found/);
  assert.throws(() => normalizePublicFeed('{"status":"error"}', publisher), /rss2json could not load this feed\./);
});

test("malformed provider payloads never turn into a successful empty subscription", () => {
  for (const data of [null,[],{}, {status:"ok"}, {status:"ok",feed:null,items:[]},
    {status:"ok",feed:[],items:[]}, {status:"ok",feed:{},items:{}}, {status:"ok",feed:{},items:[null]},
    {status:"ok",feed:{},items:["article"]}]) {
    assert.throws(() => normalizePublicFeed(JSON.stringify(data), publisher), /invalid feed response/);
  }
  assert.throws(() => normalizePublicFeed("not JSON", publisher), /invalid JSON/);
  assert.throws(() => normalizePublicFeed({}, publisher), /invalid feed response/);
  for (const url of ["javascript:alert(1)","https://user:secret@publisher.example/feed","not a URL"]) {
    assert.throws(() => normalizePublicFeed('{"status":"ok","feed":{},"items":[]}', url), /publisher feed URL/);
  }
});

test("invalid optional URLs and enclosure values are omitted without touching body markup", () => {
  const feed = normalize([{guid:"one",link:"javascript:alert(1)",thumbnail:"data:image/svg+xml,test",content:"<a href='javascript:alert(1)'>Body</a>",
    enclosure:[{link:"https://media.example/file",length:"-1",duration:"NaN"},{link:"javascript:alert(1)"}]}],
    {link:"https://user:secret@publisher.example/",image:"file:///tmp/image"});
  assert.equal("home_page_url" in feed, false);
  assert.equal("icon" in feed, false);
  assert.equal("url" in feed.items[0], false);
  assert.equal("image" in feed.items[0], false);
  assert.equal(feed.items[0].content_html, "<a href='javascript:alert(1)'>Body</a>");
  assert.deepEqual(feed.items[0].attachments, [{url:"https://media.example/file",mime_type:"application/octet-stream"}]);
});

test("provider results are not capped at ten items and retain the existing 5000-item import bound", () => {
  const items = Array.from({length:5001}, (_, i) => ({guid:`item-${i}`,content:`<p>${i}</p>`}));
  const feed = normalize(items);
  assert.equal(feed.items.length, 5000);
  assert.equal(feed.items[99].id, "item-99");
  assert.equal(feed.items[4999].content_html, "<p>4999</p>");
});
