/* global React */
const { useState: useStateC, useEffect: useEffectC } = React;

/* ============================================================
   HOME
   ============================================================ */
function HomePage() {
  return (
    <main>
      <section className="container hero">
        <figure className="hero-shot">
          <img src="assets/screen-01-corvette.png" alt="Corvette beside an asteroid, jumpgate above — in-engine screenshot of Solarance: Beginnings" />
          <figcaption>
            <span className="cap-no">▸</span>
            <span className="cap-t">Corvette beside a low-quality asteroid · Outpost Echo's jumpgate above.</span>
            <span className="cap-meta">in-engine · pre-alpha · v0.4.0</span>
          </figcaption>
        </figure>

        <div className="hero-copy">
          <div className="kicker accent-bloom">▸ Pre-Alpha · Public Devlog · Pilot enrollment closed</div>
          <img src="assets/solarance-logo.png" alt="Karl Nyborg's Solarance: Beginnings" className="hero-logo" />
          <p className="pitch">
            A cozy, persistent, co-op space sandbox.<br/>
            For pilots who loved EVE's economy but quit after the gank.<br/>
            For people who play Stardew on the couch and Factorio at midnight.<br/>
            A place to <em>contribute to something bigger than yourself in the time you have</em>.
          </p>
          <div className="ctaRow">
            <a className="btn primary accent-bloom" href="#/manifesto">▸ READ THE MANIFESTO</a>
            <a className="btn" href="#/devlog">DEVLOG</a>
            <a className="btn ghost" href="#/map">SECTOR MAP</a>
          </div>
          <div style={{ marginTop: 22, display: "flex", gap: 8, flexWrap: "wrap" }}>
            <span className="tag warn">⚠ Not playable yet</span>
            <span className="tag dim">Target: 5–10 concurrent pilots at MVP</span>
            <span className="tag dim">Solo dev · ADHD · 2 kids</span>
          </div>
        </div>
      </section>

      {/* TICKER */}
      <div className="ticker">
        <div className="track">
          <span>▸ MVP loop: <b>FIND → EXTRACT → HAUL → CONTRIBUTE → WATCH IT GROW</b></span>
          <span>▸ <b>ZERO COMBAT</b> in MVP — by design, not by accident</span>
          <span>▸ <b>1 SYSTEM · 10 SECTORS · 2 FACTIONS</b> at launch</span>
          <span>▸ Your progress persists. The galaxy ticks while you sleep.</span>
          <span>▸ Built solo, in evenings, after the kids go down ✶</span>
          <span>▸ Discord: 1 channel · 1 devlog post / month · no hype</span>
        </div>
      </div>

      {/* WHO IT'S FOR */}
      <section className="container">
        <div className="section-head">
          <div className="num accent-bloom">01</div>
          <div className="meta">
            <div className="kicker">Player target</div>
            <h2>We are making this game for one type of person.</h2>
          </div>
        </div>
        <div className="persona">
          <div className="pic">
            <div style={{ textAlign: "center" }}>
              ASCII PORTRAIT<br/>
              <span style={{ fontSize: 32, color: "var(--accent)" }}>{`( o_o )`}</span><br/>
              [ drop a placeholder ]
            </div>
          </div>
          <div>
            <div className="kicker">// pilot.archetype</div>
            <h3 style={{ marginTop: 4 }}>
              “David,” 38.
              <span style={{ marginLeft: 10, fontSize: 11, letterSpacing: ".18em", color: "var(--accent)", textTransform: "uppercase", fontFamily: "var(--font-mono)", verticalAlign: "middle" }}>— not a real pilot. an archetype.</span>
            </h3>
            <p style={{ color: "var(--fg-dim)", marginTop: 8 }}>
              We've built a composite. Call him David. IT Systems Analyst. Married, two young kids.
              Mentally drained after work. Library: No Man's Sky (peaceful), Factorio, Dyson Sphere,
              OSRS, Melvor Idle. Eight years ago he loved EVE Online for its economy, and quit after
              a PvP gank destroyed a month of work. He has never gone back.
            </p>
            <p style={{ color: "var(--fg-muted)", marginTop: 8, fontSize: 13 }}>
              You don't have to <em>be</em> David. If any of that rang a bell — the time-poor
              calm, the love of slow systems, the EVE scar tissue that never quite healed —
              there's a corvette here with your name on it.
            </p>
            <div className="qquote accent-bloom" style={{ marginTop: 18 }}>
              Solarance is for the kind of player EVE traumatized and lost.
            </div>
            <div style={{ marginTop: 18, display: "flex", gap: 8, flexWrap: "wrap" }}>
              <span className="tag">Sessions: 20 min ↔ Saturday</span>
              <span className="tag">Wants calm, not adrenaline</span>
              <span className="tag">Must not feel behind on return</span>
            </div>
          </div>
        </div>
      </section>

      {/* MVP SCOPE */}
      <section className="container">
        <div className="section-head">
          <div className="num accent-bloom">02</div>
          <div className="meta">
            <div className="kicker">MVP scope · what is actually being built</div>
            <h2>The honest list.</h2>
          </div>
        </div>

        <div className="grid-2">
          <div className="bracket"><span className="br-tr" /><span className="br-bl" />
            <div className="kicker" style={{ color: "var(--green)" }}>IN MVP</div>
            <h3 style={{ marginTop: 10 }}>What you can do</h3>
            <div className="row-list" style={{ marginTop: 14 }}>
              <div className="row"><span className="pip" /><span className="k">FIND</span><span className="v">Hand-placed asteroid fields</span></div>
              <div className="row"><span className="pip" /><span className="k">EXTRACT</span><span className="v">Simple mining beam, no minigame</span></div>
              <div className="row"><span className="pip" /><span className="k">HAUL</span><span className="v">Fly back to a partially built station</span></div>
              <div className="row"><span className="pip" /><span className="k">CONTRIBUTE</span><span className="v">Deposit into the construction pool</span></div>
              <div className="row"><span className="pip" /><span className="k">WATCH</span><span className="v">The station grows. Other pilots add too.</span></div>
            </div>
          </div>

          <div className="bracket" style={{ borderColor: "color-mix(in oklch, var(--warn) 35%, var(--line))" }}>
            <span className="br-tr" /><span className="br-bl" />
            <div className="kicker" style={{ color: "var(--warn)" }}>NOT IN MVP · ON PURPOSE</div>
            <h3 style={{ marginTop: 10 }}>What we are <em>not</em> building yet</h3>
            <p style={{ color: "var(--fg-muted)", fontSize: 12, marginTop: 8 }}>
              These belong in Future Vision. They get built when the core loop earns them —
              not before.
            </p>
            <div className="row-list" style={{ marginTop: 8 }}>
              <div className="row"><span className="pip" style={{background:"var(--warn)"}}/><span className="v">Combat of any kind</span></div>
              <div className="row"><span className="pip" style={{background:"var(--warn)"}}/><span className="v">Multiple ships / ship specializations</span></div>
              <div className="row"><span className="pip" style={{background:"var(--warn)"}}/><span className="v">Player markets / trading</span></div>
              <div className="row"><span className="pip" style={{background:"var(--warn)"}}/><span className="v">Procedural sectors, wormholes, anomalies</span></div>
              <div className="row"><span className="pip" style={{background:"var(--warn)"}}/><span className="v">Pirate NPCs, civilian AI</span></div>
              <div className="row"><span className="pip" style={{background:"var(--warn)"}}/><span className="v">Player orgs, royalty systems</span></div>
              <div className="row"><span className="pip" style={{background:"var(--warn)"}}/><span className="v">Free Trade Union · Independent Worlds Alliance · Vancellan</span></div>
            </div>
          </div>
        </div>
      </section>

      {/* PLATFORM / CLIENT */}
      <section className="container">
        <div className="section-head">
          <div className="num accent-bloom">03</div>
          <div className="meta">
            <div className="kicker">How you'll actually play it</div>
            <h2>Native Rust client. Download and run.</h2>
          </div>
        </div>

        <div className="grid-2">
          <div className="panel">
            <div className="head">
              <span className="dot" /> <span>MVP CLIENT · ON THE ROADMAP</span>
              <span style={{ flex: 1 }} />
              <span className="tag accent">RUST</span>
            </div>
            <div className="body">
              <h3 style={{ marginTop: 0, color: "var(--fg)" }}>A small executable. Windows / Linux first.</h3>
              <p style={{ color: "var(--fg-dim)", marginTop: 8 }}>
                The MVP client is a native binary built in Rust. You'll download it,
                run it, and connect to the dev SpacetimeDB module. No launcher, no
                account system beyond a SpacetimeDB anonymous identity.
              </p>
              <hr />
              <div className="spec"><div className="k">Language</div><div className="v">Rust</div></div>
              <div className="spec"><div className="k">Distribution</div><div className="v">Direct download (no Steam yet)</div></div>
              <div className="spec"><div className="k">Platforms</div><div className="v">Windows, Linux. macOS if it's cheap.</div></div>
              <div className="spec"><div className="k">Auth</div><div className="v">SpacetimeDB identity (no account)</div></div>
            </div>
          </div>

          <div className="panel outline" style={{ borderStyle: "dashed" }}>
            <div className="head">
              <span className="dot" style={{ background: "var(--fg-muted)" }} /> <span>WEB CLIENT · UNDER CONSIDERATION</span>
              <span style={{ flex: 1 }} />
              <span className="tag warn">NOT ON ROADMAP</span>
            </div>
            <div className="body">
              <h3 style={{ marginTop: 0, color: "var(--fg-dim)" }}>“Can I just play in a browser?”</h3>
              <p style={{ color: "var(--fg-dim)", marginTop: 8 }}>
                We hear you. A web client is on the table for discussion — but it's
                <b style={{ color: "var(--warn)" }}> not on the roadmap </b>
                until the native MVP loop is proven fun. One client, done well, first.
              </p>
              <p style={{ color: "var(--fg-muted)", fontSize: 13, marginTop: 8 }}>
                Want to make the case for a browser build? Or against it? The Discord is open.
              </p>
              <div style={{ marginTop: 14, display: "flex", gap: 10, flexWrap: "wrap" }}>
                <a className="btn primary" href="https://discord.solarance-beginnings.com" target="_blank" rel="noopener">
                  discord.solarance-beginnings.com →
                </a>
                <a className="btn ghost" href="#/roadmap">SEE THE ROADMAP</a>
              </div>
            </div>
          </div>
        </div>

        <p style={{ color: "var(--fg-muted)", fontSize: 12, marginTop: 18, maxWidth: "70ch" }}>
          ▸ The website you're reading is <em>just a website</em>. The System Map pulls
          live state from the public SpacetimeDB module — but the cockpit is the native client.
        </p>
      </section>

      {/* DAVID'S SESSION */}
      <section className="container">
        <div className="section-head">
          <div className="num accent-bloom">04</div>
          <div className="meta">
            <div className="kicker">What a typical MVP session looks like</div>
            <h2>15 minutes. Then bath time.</h2>
          </div>
        </div>
        <div className="terminal">
          <div><span className="prompt">▸</span> <span className="said">David logs in. He sees:</span></div>
          <div style={{ paddingLeft: 16, color: "var(--accent)" }}>
            “Welcome back. Outpost Echo (Rediar construction site) is <b>34%</b> complete.
            3 pilots contributed since your last login. You have 240 units of iron ore in storage.”
          </div>
          <hr style={{ borderTop: "1px dashed var(--line)" }} />
          <div><span className="prompt">▸</span> He flies his corvette to the asteroid sector, mines for 15 minutes.</div>
          <div><span className="prompt">▸</span> He hauls the ore back to the construction site and deposits it.</div>
          <div><span className="prompt">▸</span> The progress bar ticks up. Another pilot's ship docks while he's there.</div>
          <div><span className="prompt">▸</span> They don't talk. They both know what they're doing.</div>
          <div><span className="prompt">▸</span> He logs off.</div>
          <hr style={{ borderTop: "1px dashed var(--line)" }} />
          <div style={{ color: "var(--fg-muted)" }}>
            Twenty minutes. A small thing built. Then — dinner, bath time, sleep.
            Tomorrow night the station's a little further along, and so are you.
          </div>
        </div>

        {/* time-skip connector */}
        <div className="time-skip" role="separator" aria-label="four days later" style={{ maxWidth: "82ch", marginLeft: "auto", marginRight: "auto" }}>
          <span className="line" />
          <span className="label">▸ four days later. dinner over, kids asleep. ▸</span>
          <span className="line" />
        </div>

        {/* welcome-back mockup — the payoff */}
        <div className="panel" style={{ maxWidth: "82ch", margin: "0 auto" }}>
          <div className="head">
            <span className="dot" /> <span>WELCOME-BACK SCREEN</span>
            <span style={{ flex: 1 }} />
            <span className="tag dim">no AI · no LLM · just receipts</span>
          </div>
          <div className="body">
            <p style={{ color: "var(--fg-dim)", maxWidth: "62ch", marginTop: 0, marginBottom: 14 }}>
              This is what greets him next time. No login streak. No daily quest.
              Just a list of what the galaxy did while he was away, scoped to what he cares about.
            </p>
            <div className="terminal">
              <div><span className="prompt">$</span> ssh pilot@solarance-beginnings.com</div>
              <div style={{ marginTop: 8 }}>
                <span className="prompt">solarance ▸ </span>
                <span className="said">Welcome back, Pilot.</span>
              </div>
              <div>It has been <b style={{color:"var(--accent)"}}>4d 11h</b> since your last dock.</div>
              <hr style={{ borderTop: "1px dashed var(--line)" }} />
              <div className="said">▸ Outpost Echo (Rediar) — construction <b style={{color:"var(--accent)"}}>52%</b> <span style={{color:"var(--green)"}}>(+18% since your last visit)</span></div>
              <div style={{ color: "var(--fg-muted)" }}>  7 pilots contributed while you were away. Top contributor: <span style={{color:"var(--fg-dim)"}}>cmdr_helga</span> · 412 units</div>
              <div className="said">▸ Your storage: <b style={{color:"var(--accent)"}}>0</b> iron ore <span style={{color:"var(--fg-muted)"}}>(fully delivered last session)</span></div>
              <div className="said">▸ 1 sector notification — &nbsp;<span style={{color:"var(--warn)"}}>[FACTION]</span> &nbsp;Outpost Echo nearing 50% milestone.</div>
              <div className="said">▸ 1 sector notification — &nbsp;<span style={{color:"var(--rediar)"}}>[SYSTEM]</span> &nbsp;Quiet Belt mining yields up.</div>
              <hr style={{ borderTop: "1px dashed var(--line)" }} />
              <div style={{ color: "var(--fg-muted)" }}>Your progress waited for you. It will keep waiting.</div>
              <div style={{ marginTop: 10 }}>
                <span className="prompt">▸</span> <span className="cursor" />
              </div>
            </div>
            <p style={{ color: "var(--fg-muted)", fontSize: 12, marginTop: 14 }}>
              ▸ Mockup. The MVP welcome-back is text-only — visuals come later, if they earn their place.
            </p>
          </div>
        </div>
      </section>

      {/* TWO FACTIONS */}
      <section className="container">
        <div className="section-head">
          <div className="num accent-bloom">05</div>
          <div className="meta">
            <div className="kicker">// pretenders & doubters</div>
            <h2>Two banners. A system that disagrees about its own name.</h2>
          </div>
        </div>
        <p style={{ color: "var(--fg-dim)", maxWidth: "72ch", marginBottom: 10 }}>
          Two factions are playable in the MVP. They disagree about something fundamental:
          the world the <b style={{ color: "var(--lrak)" }}>Lrak Combine</b> rules from —
          which the Combine calls humanity's homeworld, the seat of its throne —
          is, the <b style={{ color: "var(--rediar)" }}>Rediar Federation</b> insist,
          not Earth at all.
        </p>
        <p style={{ color: "var(--fg-muted)", maxWidth: "72ch", marginBottom: 22, fontSize: 13 }}>
          The Rediar's first colony ship left before the Combine even existed.
          Out past the edge of charted space it stumbled into alien ruins, and on those
          ruins were inscriptions referring to <em>early humans</em>. The implication is
          inescapable and unwelcome: humanity is far from where it began, has been for
          a very long time, and nobody alive remembers the way back. For most pilots —
          on either side — Earth is a myth.
        </p>

        <div className="grid-2">
          <div className="faction-card lrak">
            <div className="crest">L</div>
            <div className="kicker" style={{ color: "var(--lrak)" }}>FACTION · TIER 1 · PLAYABLE</div>
            <h3>Lrak Combine</h3>
            <p style={{ color: "var(--fg)" }}>The pretenders to a throne.</p>
            <p style={{ color: "var(--fg-dim)" }}>
              The Combine rules from the world it calls humanity's cradle, and crowns its
              rulers there. Whether the cradle is actually Earth is, to a Lrak citizen,
              a settled question — and an impolite one.
            </p>
            <hr />
            <div className="spec"><div className="k">Color</div><div className="v">Red</div></div>
            <div className="spec"><div className="k">Capital</div><div className="v">Lrakhold (MVP)</div></div>
            <div className="spec"><div className="k">Joinable</div><div className="v">Yes</div></div>
          </div>

          <div className="faction-card rediar">
            <div className="crest">R</div>
            <div className="kicker" style={{ color: "var(--rediar)" }}>FACTION · TIER 1 · PLAYABLE</div>
            <h3>Rediar Federation</h3>
            <p style={{ color: "var(--fg)" }}>The doubters with the evidence.</p>
            <p style={{ color: "var(--fg-dim)" }}>
              A federation built from a colony ship that left before the Combine existed.
              They know what the Combine denies. They don't know where Earth is, or when
              humanity left it — only that it wasn't here, and it wasn't recent.
            </p>
            <hr />
            <div className="spec"><div className="k">Color</div><div className="v">Blue</div></div>
            <div className="spec"><div className="k">Capital</div><div className="v">Outpost Echo (MVP)</div></div>
            <div className="spec"><div className="k">Joinable</div><div className="v">Yes</div></div>
          </div>
        </div>

        <div className="bracket" style={{ marginTop: 22, borderStyle: "dashed" }}>
          <span className="br-tr" /><span className="br-bl" />
          <div style={{ display: "flex", gap: 18, alignItems: "baseline", flexWrap: "wrap" }}>
            <div className="kicker" style={{ color: "var(--accent)" }}>// five more in the wings</div>
            <p style={{ color: "var(--fg-dim)", margin: 0, flex: 1, minWidth: 320 }}>
              Seven factions exist in the world's lore — pieced together over a decade of notes.
              Two are playable at MVP. The other five wait for v1.0 onward, when the systems
              that make a faction <em>matter</em> (rep, research, faction-weighted economy) are
              built to receive them. Names withheld. Speculation welcomed in the Discord.
            </p>
          </div>
        </div>
      </section>

      {/* SCREENSHOTS / IN-GAME */}
      <section className="container">
        <div className="section-head">
          <div className="num accent-bloom">06</div>
          <div className="meta">
            <div className="kicker">From the dev build</div>
            <h2>Handcrafted. Hand-pixeled. Hand-flown.</h2>
          </div>
        </div>
        <p style={{ color: "var(--fg-dim)", maxWidth: "70ch", marginBottom: 18 }}>
          The ship sprites in our cockpit were drawn pixel-by-pixel in 2010, by the same person
          still flying them sixteen years later. Asteroids and stations are 3D-rendered in-house.
          No asset packs, no marketplace ships, no AI-generated anything.
        </p>
        <p style={{ color: "var(--fg-muted)", maxWidth: "70ch", marginBottom: 28, fontSize: 13 }}>
          If you grew up on <i>EVE</i>, <i>Freelancer</i>, <i>Escape Velocity</i>, or any number of
          late-night downloaded shareware space games — welcome back. We're flying in the same
          lineage as <i>Old School RuneScape</i>, <i>Dwarf Fortress</i>, and <i>Caves of Qud</i>:
          games where the world is more important than the polygon count.
        </p>

        <div className="screens">
          <figure className="screen wide">
            <img src="assets/screen-01-corvette.png" alt="Corvette near an asteroid and jumpgate, in-engine screenshot" />
            <figcaption>
              <span className="cap-no">01</span>
              <span className="cap-t">Corvette beside a low-quality asteroid, station jumpgate above.</span>
              <span className="cap-meta">in-engine · pre-alpha build</span>
            </figcaption>
          </figure>

          <figure className="screen tall">
            <img src="assets/screen-02-pixel-fleet.png" alt="Hand-pixeled ship and station sprites on a nebula background" />
            <figcaption>
              <span className="cap-no">02</span>
              <span className="cap-t">Sprite sheet — sixteen years of hand-pixeled assets.</span>
              <span className="cap-meta">art · 2010 → present</span>
            </figcaption>
          </figure>

          <figure className="screen wide">
            <img src="assets/screen-03-station-menu.png" alt="Station Panel showing the manufacturing module, cargo tree, and server chat log" />
            <figcaption>
              <span className="cap-no">03</span>
              <span className="cap-t">Docked at a Trading Station. Basic Factory module, cargo tree, sector chat.</span>
              <span className="cap-meta">UI · pre-alpha · subject to change</span>
            </figcaption>
          </figure>
        </div>

        <p style={{ color: "var(--fg-muted)", fontSize: 12, marginTop: 18 }}>
          ▸ These are real frames from the current dev build. Not concept art. Not mockups.
        </p>
      </section>
    </main>
  );
}

/* ============================================================
   MANIFESTO
   ============================================================ */
function ManifestoPage() {
  return (
    <main className="container" style={{ padding: "60px 18px" }}>
      <div className="kicker accent-bloom">▸ Mission · single source of truth</div>
      <h1 style={{ marginTop: 8, color: "var(--fg)" }}>Manifesto</h1>
      <p style={{ marginTop: 18, fontSize: 16, color: "var(--fg-dim)", maxWidth: "62ch" }}>
        Solarance: Beginnings is a cozy persistent space MMO for adults with jobs.
        Contribute to something bigger than yourself in the time you have.
        Your progress will be waiting for you.
      </p>

      <hr />

      <div className="grid-2" style={{ gap: 40 }}>
        <div>
          <div className="kicker">// 01</div>
          <h2 style={{ marginTop: 8 }}>One pillar.</h2>
          <p style={{ marginTop: 12, color: "var(--fg-dim)" }}>
            <b>Expansion (building) is primary.</b> Everything else collapses into service.
            Mining produces resources for building. Trading is hauling ore to a construction site.
            Exploration is stubbed. Combat is absent. The whole MVP is one verb: <em>contribute</em>.
          </p>
        </div>
        <div>
          <div className="kicker">// 02</div>
          <h2 style={{ marginTop: 8 }}>Respect the player's time.</h2>
          <p style={{ marginTop: 12, color: "var(--fg-dim)" }}>
            Sessions are 20 minutes. Sometimes a Saturday. Sometimes nothing for a week.
            The game must wait for the player without making them feel behind.
            No login streaks. No daily quests. No FOMO.
          </p>
        </div>
        <div>
          <div className="kicker">// 03</div>
          <h2 style={{ marginTop: 8 }}>Cozy is permanent.</h2>
          <p style={{ marginTop: 12, color: "var(--fg-dim)" }}>
            Combat may appear in later versions — as environmental weather, never as a
            required activity. Core sectors stay safe. Forever. If PvP ever ships, it lives
            in designated lawless space, opt-in, and you'll never wake up to find your stuff gone.
          </p>
        </div>
        <div>
          <div className="kicker">// 04</div>
          <h2 style={{ marginTop: 8 }}>Honest scope.</h2>
          <p style={{ marginTop: 12, color: "var(--fg-dim)" }}>
            We post one devlog a month. We don't promise features that don't exist.
            A trailer comes when there's a game to trailer.
            The Discord is small, and that's the point — join us early or join us late.
          </p>
        </div>
      </div>

      <hr />

      <div className="kicker">// the promise</div>
      <div className="qquote accent-bloom" style={{ marginTop: 12 }}>
        We'll keep the MVP small. <br/>
        We'll be honest about what's in it and what isn't. <br/>
        Your progress will be waiting for you.
      </div>

      <div style={{ marginTop: 28, display: "flex", gap: 12, flexWrap: "wrap" }}>
        <a className="btn primary" href="#/roadmap">▸ SEE THE ROADMAP</a>
        <a className="btn" href="#/devlog">DEVLOG</a>
      </div>
    </main>
  );
}

/* ============================================================
   ROADMAP
   ============================================================ */
function RoadmapPage() {
  return (
    <main className="container" style={{ padding: "60px 18px" }}>
      <div className="kicker accent-bloom">▸ Tiered. Future Vision is uncommitted by design.</div>
      <h1>Roadmap</h1>
      <p style={{ color: "var(--fg-dim)", marginTop: 14, maxWidth: "62ch" }}>
        Anything below MVP is aspirational. Tiers are not on a calendar.
        A tier ships when the previous tier is proven fun by real pilots.
      </p>

      <div className="tl-track" style={{ marginTop: 36 }}>
        <div className="tl-row">
          <div className="dot" />
          <div className="stage accent-bloom">PHASE 0 · MVP — IN PROGRESS</div>
          <h3 style={{ color: "var(--fg)" }}>Prove the loop. Two pilots, one station, one shared moment.</h3>
          <p style={{ color: "var(--fg-dim)", marginTop: 6 }}>
            One solar system. Ten hand-placed sectors. Two factions. One corvette per pilot. Find → extract → haul → contribute → watch it grow.
            Persistent stations and inventories. No combat. No NPCs. 5–10 concurrent pilots target.
          </p>
          <div style={{ marginTop: 10, display: "flex", flexWrap: "wrap", gap: 8 }}>
            <span className="tag accent">Movement rework</span>
            <span className="tag accent">Welcome-back screen</span>
            <span className="tag accent">2-player shared building spike</span>
            <span className="tag dim">Persistence pass</span>
          </div>
        </div>

        <div className="tl-row future">
          <div className="dot" />
          <div className="stage">PHASE 1 · v1.0 — SYSTEMS THAT FEEL ALIVE</div>
          <h3>Persistent NPC economy. Faction rep. Mining minigame. Specialized hulls.</h3>
          <p style={{ color: "var(--fg-dim)", marginTop: 6 }}>
            Free Trade Union and Independent Worlds Alliance arrive. NPC traders visit your station while you sleep.
            Mining gets a heatmap + intensity dial. Dedicated miners, freighters, builders, explorers.
            Sub-groups within factions. A simplified royalty system that pays from nowhere — no player-funded markets yet.
          </p>
        </div>

        <div className="tl-row future">
          <div className="dot" />
          <div className="stage">PHASE 2 · v1.1 — THE LIVING COSMOS</div>
          <h3>Vancellan-touched sectors. Drones. A second solar system. Wormholes.</h3>
          <p style={{ color: "var(--fg-dim)", marginTop: 6 }}>
            Environmental pressure, not enemies. Higher yield where the bio-residue is, higher NPC patrol density too.
            Players choose the risk/reward. Anomalies become a real discipline. LLM-narrated welcome-back, gated by cost.
          </p>
        </div>

        <div className="tl-row future">
          <div className="dot" />
          <div className="stage">PHASE 3 · v2.0 — COMBAT AS OPT-IN</div>
          <h3>Playable combat for pilots who want it. Fleet command. PvP in lawless space only.</h3>
          <p style={{ color: "var(--fg-dim)", marginTop: 6 }}>
            Core sectors remain safe forever. The carebear contract is inviolable.
            Large-scale faction warfare emerges from pilots who choose to fight, in the places where it's allowed.
          </p>
        </div>
      </div>
    </main>
  );
}

/* ============================================================
   COMMUNITY / Pilot's Lounge
   ============================================================ */
function CommunityPage() {
  return (
    <main className="container" style={{ padding: "60px 18px" }}>
      <div className="kicker accent-bloom">▸ Small. Patient. Honest.</div>
      <h1>Pilot's Lounge</h1>
      <p style={{ color: "var(--fg-dim)", marginTop: 14, maxWidth: "62ch" }}>
        A Discord with one general channel and one devlog channel. We're 40-something pilots.
        The bar is not how loud we are. The bar is whether we're still here in 18 months.
      </p>

      <div className="grid-3" style={{ marginTop: 32 }}>
        <a className="card" href="#" onClick={e => e.preventDefault()} style={{ display: "block" }}>
          <div className="meta">// channel</div>
          <h3>#general</h3>
          <p style={{ color: "var(--fg-dim)", marginTop: 6 }}>Pilots chat about the game, share other space games, post screenshots of their kids' drawings.</p>
        </a>
        <a className="card" href="#" onClick={e => e.preventDefault()} style={{ display: "block" }}>
          <div className="meta">// channel</div>
          <h3>#devlog</h3>
          <p style={{ color: "var(--fg-dim)", marginTop: 6 }}>Monthly posts. Auto-mirrored to RSS. Comments encouraged.</p>
        </a>
        <a className="card" href="#" onClick={e => e.preventDefault()} style={{ display: "block" }}>
          <div className="meta">// channel</div>
          <h3>#playtest-waitlist</h3>
          <p style={{ color: "var(--fg-dim)", marginTop: 6 }}>Closed. Reopens when the two-pilot shared-building spike works.</p>
        </a>
      </div>

      <hr style={{ marginTop: 36 }} />

      <div className="grid-2" style={{ marginTop: 28 }}>
        <div className="panel">
          <div className="head"><span className="dot" /> <span>JOIN THE DISCORD</span></div>
          <div className="body">
            <p style={{ color: "var(--fg-dim)" }}>One server. No XP bots. No verification quizzes.</p>
            <a className="btn primary" href="https://discord.solarance-beginnings.com" target="_blank" rel="noopener">discord.solarance-beginnings.com →</a>
          </div>
        </div>
        <div className="panel">
          <div className="head"><span className="dot" /> <span>OPEN MAIL CHANNELS</span></div>
          <div className="body">
            <div className="spec"><div className="k">RSS</div><div className="v">solarance-beginnings.com/devlog.rss</div></div>
            <div className="spec"><div className="k">Email (low traffic)</div><div className="v">pilot@solarance-beginnings.com</div></div>
            <div className="spec"><div className="k">SpacetimeDB</div><div className="v">testnet.spacetimedb.com / solarance</div></div>
            <div className="spec"><div className="k">Source devlog</div><div className="v">github.com / solarance / beginnings</div></div>
          </div>
        </div>
      </div>
    </main>
  );
}

window.HomePage = HomePage;
window.ManifestoPage = ManifestoPage;
window.RoadmapPage = RoadmapPage;
window.CommunityPage = CommunityPage;
