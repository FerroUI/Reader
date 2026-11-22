import { httpUrl } from "./feeds.js";
export function downloadFile(name, contents, type) {
  const url = URL.createObjectURL(new Blob([contents], { type }));
  const link = document.createElement("a"); link.href = url; link.download = name;
  document.body.append(link); link.click(); link.remove();
  setTimeout(() => URL.revokeObjectURL(url), 30000);
}
export function pickFile(format, runtime) {
  const input = document.createElement("input"); input.type = "file";
  input.accept = format === "opml" ? ".opml,.xml,text/xml,application/xml" : ".json,application/json";
  input.addEventListener("change", async () => {
    try {
      const file = input.files?.[0]; if (!file) return;
      if (file.size > 64 * 1024 * 1024) throw new Error("Choose a file smaller than 64 MB.");
      const text = await file.text();
      if (format === "opml") runtime.importOpml(text); else runtime.importData(text);
      runtime.flushSave();
    } catch (error) { runtime.hostError(error.message || String(error)); }
    finally { input.remove(); }
  });
  input.addEventListener("cancel", () => input.remove());
  input.hidden = true; document.body.append(input); input.click();
}
export function openUrl(url) {
  const target = httpUrl(url);
  const link = document.createElement("a"); link.href = target; link.target = "_blank"; link.rel = "noopener noreferrer";
  document.body.append(link); link.click(); link.remove();
}
export async function copyText(text) {
  if (navigator.clipboard?.writeText) return navigator.clipboard.writeText(text);
  const input = document.createElement("textarea"); input.value = text; document.body.append(input); input.select();
  const copied = document.execCommand("copy"); input.remove();
  if (!copied) throw new Error("Copy is unavailable. Open the article to copy its address.");
}
export async function share(title, url) {
  url = httpUrl(url);
  if (navigator.share) { try { await navigator.share({ title, url }); return "Shared"; } catch (e) { if (e.name === "AbortError") return ""; throw e; } }
  await copyText(url); return "Article link copied";
}
export function email(title, url) {
  location.href = `mailto:?subject=${encodeURIComponent(title)}&body=${encodeURIComponent(httpUrl(url))}`;
}
export function printArticle(title, html, baseUrl) {
  const source = new DOMParser().parseFromString(html, "text/html");
  for (const node of source.querySelectorAll("script,style,iframe,object,embed,link,meta,form,input,button,svg,math")) node.remove();
  for (const node of source.querySelectorAll("*")) {
    for (const attribute of [...node.attributes]) {
      if (!['href', 'src', 'alt', 'title', 'colspan', 'rowspan'].includes(attribute.name)) node.removeAttribute(attribute.name);
      if (['href', 'src'].includes(attribute.name)) {
        try { node.setAttribute(attribute.name, httpUrl(attribute.value, baseUrl)); } catch { node.removeAttribute(attribute.name); }
      }
    }
  }
  const frame = document.createElement("iframe"); frame.hidden = true;
  frame.sandbox = "allow-same-origin allow-modals";
  document.body.append(frame);
  const documentCopy = frame.contentDocument;
  documentCopy.open(); documentCopy.write('<!doctype html><html><head><meta charset="utf-8"><title></title><style>body{font:14px/1.6 Arial,sans-serif;max-width:760px;margin:40px auto}img{max-width:100%}a{color:#15c}</style></head><body></body></html>'); documentCopy.close();
  documentCopy.title = title;
  const heading = documentCopy.createElement("h1"); heading.textContent = title;
  documentCopy.body.append(heading, documentCopy.importNode(source.body, true));
  frame.contentWindow.focus(); frame.contentWindow.print();
  setTimeout(() => frame.remove(), 60000);
}
