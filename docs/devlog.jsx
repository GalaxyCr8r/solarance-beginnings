/* global React */
const { useState: useStateD, useEffect: useEffectD, useMemo: useMemoD } = React;

/* ============================================================
   DEVLOG — file-driven
   ============================================================
   To add a new post:
     1. Drop a markdown file into solarance/devlog/, e.g.
        005-my-new-post.md  with frontmatter (see existing posts).
     2. Add its filename (without .md) to solarance/devlog/index.json,
        at the TOP of the array (newest first).
     3. Refresh. Title, date, tag, tags, and blurb all parse from the file.

   Frontmatter keys (all optional except title):
     slug:  string  (used in #/devlog/<slug>; defaults to filename)
     date:  YYYY-MM-DD
     tag:   short label shown next to date (e.g. "DEVLOG · 005")
     title: string  (required)
     tags:  [a, b, c]   (comma-separated inside [ ])
     blurb: string  (overrides default = first paragraph of body)

   The body supports a small, safe subset of Markdown rendered by
   a tiny built-in parser (no external dependency, works offline).
   ============================================================ */

function DevlogPage() {
  const [posts, setPosts] = useStateD([]);
  const [error, setError] = useStateD(null);
  const [loading, setLoading] = useStateD(true);
  const [openSlug, setOpenSlug] = useStateD(() => readDevlogHash());

  useEffectD(() => {
    const onHash = () => setOpenSlug(readDevlogHash());
    window.addEventListener("hashchange", onHash);
    return () => window.removeEventListener("hashchange", onHash);
  }, []);

  useEffectD(() => {
    let cancelled = false;
    (async () => {
      try {
        const manifest = await fetch("devlog/index.json").then(r => {
          if (!r.ok) throw new Error("Could not load devlog/index.json (" + r.status + ")");
          return r.json();
        });
        const loaded = await Promise.all(manifest.map(async (slug) => {
          const text = await fetch(`devlog/${slug}.md`).then(r => {
            if (!r.ok) throw new Error(`Could not load devlog/${slug}.md`);
            return r.text();
          });
          return parsePost(slug, text);
        }));
        if (!cancelled) {
          // Sort by date desc as a safety net (manifest order should already be correct)
          loaded.sort((a, b) => (b.date || "") .localeCompare(a.date || ""));
          setPosts(loaded);
          setLoading(false);
        }
      } catch (e) {
        if (!cancelled) { setError(String(e.message || e)); setLoading(false); }
      }
    })();
    return () => { cancelled = true; };
  }, []);

  const activePost = useMemoD(
    () => posts.find(p => p.slug === openSlug) || null,
    [posts, openSlug]
  );

  if (activePost) return <DevlogPostView post={activePost} all={posts} />;

  return (
    <main className="container" style={{ padding: "60px 18px" }}>
      <div className="kicker accent-bloom">▸ One post per month. Even if it's short.</div>
      <h1>Devlog</h1>
      <p style={{ color: "var(--fg-dim)", marginTop: 14, maxWidth: "62ch" }}>
        No release dates. No hype. Just receipts.
      </p>

      {loading && <DevlogLoading />}
      {error && <DevlogError msg={error} />}

      <div style={{ marginTop: 28 }}>
        {posts.map(e => (
          <article key={e.slug} className="entry">
            <div>
              <div className="date">{fmtDate(e.date)}</div>
              {e.tag && <div className="tag accent" style={{ marginTop: 8 }}>{e.tag}</div>}
            </div>
            <div>
              <h3>
                <a href={`#/devlog/${e.slug}`} style={{ color: "var(--fg)" }}>{e.title}</a>
              </h3>
              <p className="body">{e.blurb}</p>
              <div style={{ marginTop: 8, display: "flex", gap: 6, flexWrap: "wrap", alignItems: "center" }}>
                {(e.tags || []).map(t => <span key={t} className="tag dim">#{t}</span>)}
                <a href={`#/devlog/${e.slug}`} className="tag accent" style={{ marginLeft: "auto" }}>READ →</a>
              </div>
            </div>
          </article>
        ))}
      </div>

      <hr style={{ marginTop: 32 }} />
      <div className="bracket" style={{ marginTop: 20 }}>
        <span className="br-tr" /><span className="br-bl" />
        <div className="kicker">// next post</div>
        <h3 style={{ marginTop: 8 }}>Subscribe before you forget.</h3>
        <p style={{ color: "var(--fg-dim)", marginTop: 6 }}>
          New post when there's something honest to say. Drop into the Discord, or grab the RSS feed.
        </p>
        <div style={{ marginTop: 14, display: "flex", gap: 10, flexWrap: "wrap" }}>
          <a className="btn" href="https://discord.solarance-beginnings.com" target="_blank" rel="noopener">DISCORD →</a>
          <a className="btn ghost" href="#/community">PILOT'S LOUNGE</a>
        </div>
      </div>

      <details style={{ marginTop: 28, color: "var(--fg-muted)", fontSize: 12 }}>
        <summary style={{ cursor: "pointer", color: "var(--fg-dim)" }}>How do I add a new post?</summary>
        <div style={{ marginTop: 10, paddingLeft: 14, borderLeft: "1px dashed var(--line)" }}>
          <ol style={{ paddingLeft: 18 }}>
            <li>Drop a markdown file into <code>solarance/devlog/</code> — e.g. <code>005-my-post.md</code>.</li>
            <li>Open it with frontmatter at the top (see any existing post for the format).</li>
            <li>Open <code>solarance/devlog/index.json</code> and add <code>"005-my-post"</code> at the top of the list.</li>
            <li>Refresh. Done — list and individual page both populate from the file.</li>
          </ol>
        </div>
      </details>
    </main>
  );
}

/* ============================================================
   Single post view
   ============================================================ */
function DevlogPostView({ post, all }) {
  const idx = all.findIndex(p => p.slug === post.slug);
  const prev = idx >= 0 && idx < all.length - 1 ? all[idx + 1] : null; // older
  const next = idx > 0 ? all[idx - 1] : null; // newer

  return (
    <main className="container" style={{ padding: "44px 18px 60px", maxWidth: 880 }}>
      <a href="#/devlog" className="kicker" style={{ display: "inline-block", marginBottom: 24 }}>
        ← BACK TO ALL DEVLOGS
      </a>

      <div style={{ display: "flex", gap: 16, alignItems: "baseline", flexWrap: "wrap" }}>
        <div className="date" style={{ color: "var(--accent)", fontFamily: "var(--font-mono)", fontSize: 12, letterSpacing: ".22em", textTransform: "uppercase" }}>
          {fmtDate(post.date)}
        </div>
        {post.tag && <span className="tag accent">{post.tag}</span>}
      </div>

      <h1 style={{ marginTop: 14 }}>{post.title}</h1>

      {(post.tags || []).length > 0 && (
        <div style={{ marginTop: 14, display: "flex", gap: 6, flexWrap: "wrap" }}>
          {post.tags.map(t => <span key={t} className="tag dim">#{t}</span>)}
        </div>
      )}

      <hr style={{ margin: "28px 0" }} />

      <article
        className="md-body"
        dangerouslySetInnerHTML={{ __html: post.html }}
      />

      <hr style={{ margin: "40px 0 20px" }} />

      <div style={{ display: "flex", justifyContent: "space-between", gap: 18, flexWrap: "wrap" }}>
        {prev ? (
          <a href={`#/devlog/${prev.slug}`} className="card" style={{ flex: 1, minWidth: 240, textDecoration: "none" }}>
            <div className="meta">← OLDER</div>
            <div style={{ color: "var(--fg)" }}>{prev.title}</div>
          </a>
        ) : <div style={{ flex: 1 }} />}
        {next ? (
          <a href={`#/devlog/${next.slug}`} className="card" style={{ flex: 1, minWidth: 240, textDecoration: "none", textAlign: "right" }}>
            <div className="meta">NEWER →</div>
            <div style={{ color: "var(--fg)" }}>{next.title}</div>
          </a>
        ) : <div style={{ flex: 1 }} />}
      </div>
    </main>
  );
}

/* ============================================================
   Helpers
   ============================================================ */
function readDevlogHash() {
  const h = window.location.hash.replace(/^#\/?/, "");
  const m = h.match(/^devlog\/(.+)$/);
  return m ? m[1] : null;
}

function fmtDate(iso) {
  if (!iso) return "";
  // expects YYYY-MM-DD
  return iso.replace(/-/g, " · ");
}

function DevlogLoading() {
  return (
    <div className="terminal" style={{ marginTop: 24, maxWidth: 480 }}>
      <div><span style={{ color: "var(--accent)" }}>▸</span> fetching devlog/index.json...</div>
      <div style={{ color: "var(--fg-muted)" }}>  parsing markdown frontmatter...</div>
      <div><span style={{ color: "var(--accent)" }}>▸</span> <span className="cursor" /></div>
    </div>
  );
}

function DevlogError({ msg }) {
  return (
    <div className="bracket" style={{ marginTop: 24, borderColor: "color-mix(in oklch, var(--warn) 50%, var(--line))" }}>
      <span className="br-tr" /><span className="br-bl" />
      <div className="kicker" style={{ color: "var(--warn)" }}>// fetch error</div>
      <h3 style={{ marginTop: 8 }}>Couldn't load the devlog index.</h3>
      <p style={{ color: "var(--fg-dim)", marginTop: 6, fontFamily: "var(--font-mono)", fontSize: 12 }}>
        {msg}
      </p>
      <p style={{ color: "var(--fg-muted)", marginTop: 6, fontSize: 12 }}>
        If you're opening <code>index.html</code> directly from <code>file://</code>,
        the browser blocks <code>fetch()</code> for local files. Serve this folder over http
        (e.g. <code>python -m http.server</code>, <code>npx serve</code>, or any static host) and reload.
      </p>
    </div>
  );
}

/* ============================================================
   Markdown + frontmatter parser
   ============================================================
   No dependency. Supports:
     - YAML-ish frontmatter at top: --- key: value --- (one level)
     - # / ## / ### headings
     - paragraphs
     - **bold**  *italic*  `code`
     - [text](url) links (target _blank for external)
     - > blockquotes
     - ``` fenced code blocks
     - - and 1. lists
     - --- horizontal rule
     - automatic blurb = first paragraph

   Escapes all HTML for safety. We treat post files as trusted-but-careful.
   ============================================================ */
function parsePost(slug, raw) {
  const { frontmatter, body } = splitFrontmatter(raw);

  const html = renderMarkdown(body);
  // first non-heading paragraph for the blurb fallback
  const blurb = frontmatter.blurb || extractFirstParagraph(body) || "";

  return {
    slug:  frontmatter.slug || slug,
    date:  frontmatter.date || "",
    tag:   frontmatter.tag || "",
    title: frontmatter.title || slug,
    tags:  parseTagList(frontmatter.tags),
    blurb,
    html,
  };
}

function splitFrontmatter(text) {
  const lines = text.replace(/\r\n/g, "\n").split("\n");
  if (lines[0]?.trim() !== "---") return { frontmatter: {}, body: text };
  const closeIdx = lines.findIndex((l, i) => i > 0 && l.trim() === "---");
  if (closeIdx === -1) return { frontmatter: {}, body: text };
  const fmLines = lines.slice(1, closeIdx);
  const body = lines.slice(closeIdx + 1).join("\n").replace(/^\s+/, "");
  const frontmatter = {};
  for (const line of fmLines) {
    const m = line.match(/^([a-zA-Z0-9_-]+)\s*:\s*(.*)$/);
    if (m) frontmatter[m[1]] = m[2].trim();
  }
  return { frontmatter, body };
}

function parseTagList(raw) {
  if (!raw) return [];
  // accept  [a, b, c]   or   a, b, c
  const inner = raw.replace(/^\[|\]$/g, "").trim();
  if (!inner) return [];
  return inner.split(",").map(s => s.trim().replace(/^["']|["']$/g, "")).filter(Boolean);
}

function extractFirstParagraph(body) {
  const blocks = body.split(/\n\s*\n/);
  for (const block of blocks) {
    const t = block.trim();
    if (!t) continue;
    if (t.startsWith("#") || t.startsWith(">") || t.startsWith("```") || t.startsWith("---")) continue;
    // strip markdown emphasis for the blurb
    return t
      .replace(/\n/g, " ")
      .replace(/`([^`]+)`/g, "$1")
      .replace(/\*\*([^*]+)\*\*/g, "$1")
      .replace(/\*([^*]+)\*/g, "$1")
      .replace(/\[([^\]]+)\]\([^)]+\)/g, "$1");
  }
  return "";
}

/* tiny markdown renderer */
function escapeHTML(s) {
  return s
    .replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;").replace(/'/g, "&#39;");
}

function renderInline(s) {
  // escape first, then re-introduce safe markdown
  let out = escapeHTML(s);
  // inline code
  out = out.replace(/`([^`]+)`/g, (_m, c) => `<code>${c}</code>`);
  // bold
  out = out.replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>");
  // italic
  out = out.replace(/\*([^*]+)\*/g, "<em>$1</em>");
  // links — [text](url)
  out = out.replace(/\[([^\]]+)\]\(([^)\s]+)\)/g, (_m, txt, url) => {
    const safe = url.replace(/"/g, "%22");
    const external = /^https?:/i.test(safe);
    return `<a href="${safe}"${external ? ' target="_blank" rel="noopener"' : ""}>${txt}</a>`;
  });
  return out;
}

function renderMarkdown(src) {
  const lines = src.replace(/\r\n/g, "\n").split("\n");
  const out = [];
  let i = 0;

  const flushParagraph = (buf) => {
    if (!buf.length) return;
    out.push(`<p>${renderInline(buf.join(" ").trim())}</p>`);
  };

  let para = [];

  while (i < lines.length) {
    const line = lines[i];

    // fenced code
    if (/^```/.test(line)) {
      flushParagraph(para); para = [];
      const lang = line.replace(/^```/, "").trim();
      i++;
      const codeBuf = [];
      while (i < lines.length && !/^```/.test(lines[i])) { codeBuf.push(lines[i]); i++; }
      i++; // skip closing fence
      out.push(`<pre class="md-pre"><code${lang ? ` data-lang="${escapeHTML(lang)}"` : ""}>${escapeHTML(codeBuf.join("\n"))}</code></pre>`);
      continue;
    }

    // hr
    if (/^---\s*$/.test(line)) {
      flushParagraph(para); para = [];
      out.push("<hr/>");
      i++; continue;
    }

    // headings
    const hm = line.match(/^(#{1,4})\s+(.*)$/);
    if (hm) {
      flushParagraph(para); para = [];
      const level = hm[1].length;
      out.push(`<h${level + 1}>${renderInline(hm[2])}</h${level + 1}>`);
      i++; continue;
    }

    // blockquote (one or more lines starting with >)
    if (/^>\s?/.test(line)) {
      flushParagraph(para); para = [];
      const qBuf = [];
      while (i < lines.length && /^>\s?/.test(lines[i])) {
        qBuf.push(lines[i].replace(/^>\s?/, ""));
        i++;
      }
      out.push(`<blockquote>${renderInline(qBuf.join(" "))}</blockquote>`);
      continue;
    }

    // unordered list
    if (/^[-*]\s+/.test(line)) {
      flushParagraph(para); para = [];
      const items = [];
      while (i < lines.length && /^[-*]\s+/.test(lines[i])) {
        items.push(lines[i].replace(/^[-*]\s+/, ""));
        i++;
      }
      out.push(`<ul>${items.map(it => `<li>${renderInline(it)}</li>`).join("")}</ul>`);
      continue;
    }

    // ordered list
    if (/^\d+\.\s+/.test(line)) {
      flushParagraph(para); para = [];
      const items = [];
      while (i < lines.length && /^\d+\.\s+/.test(lines[i])) {
        items.push(lines[i].replace(/^\d+\.\s+/, ""));
        i++;
      }
      out.push(`<ol>${items.map(it => `<li>${renderInline(it)}</li>`).join("")}</ol>`);
      continue;
    }

    // blank line → paragraph break
    if (/^\s*$/.test(line)) {
      flushParagraph(para); para = [];
      i++; continue;
    }

    // accumulate paragraph
    para.push(line);
    i++;
  }
  flushParagraph(para);

  return out.join("\n");
}

window.DevlogPage = DevlogPage;
