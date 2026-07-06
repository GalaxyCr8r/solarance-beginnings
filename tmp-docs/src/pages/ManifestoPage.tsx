export function ManifestoPage() {
  return (
    <main className="container prose">
      <div className="kicker">▸ MISSION · SINGLE SOURCE OF TRUTH</div>
      <h1>Manifesto</h1>
      <p className="lede">
        Solarance: Beginnings is a cozy persistent space MMO for adults with
        jobs. Contribute to something bigger than yourself in the time you
        have. Your progress will be waiting for you.
      </p>

      <hr />

      <div className="grid-2">
        <div>
          <div className="kicker">// 01</div>
          <h2>One pillar.</h2>
          <p>
            <b>Expansion (building) is primary.</b> Everything else collapses
            into service. Mining produces resources for building. Trading is
            hauling ore to a construction site. Exploration is stubbed. Combat
            is absent. The whole MVP is one verb: <em>contribute</em>.
          </p>
        </div>
        <div>
          <div className="kicker">// 02</div>
          <h2>Respect the player's time.</h2>
          <p>
            Sessions are 20 minutes. Sometimes a Saturday. Sometimes nothing
            for a week. The game must wait for the player without making them
            feel behind. No login streaks. No daily quests. No FOMO.
          </p>
        </div>
        <div>
          <div className="kicker">// 03</div>
          <h2>Cozy is permanent.</h2>
          <p>
            Combat may appear in later versions — as environmental weather,
            never as a required activity. Core sectors stay safe. Forever. If
            PvP ever ships, it lives in designated lawless space, opt-in, and
            you'll never wake up to find your stuff gone.
          </p>
        </div>
        <div>
          <div className="kicker">// 04</div>
          <h2>Honest scope.</h2>
          <p>
            We post one devlog a month. We don't promise features that don't
            exist. A trailer comes when there's a game to trailer. The Discord
            is small, and that's the point — join us early or join us late.
          </p>
        </div>
      </div>

      <hr />

      <div className="kicker">// the promise</div>
      <blockquote className="qquote">
        We'll keep the MVP small.
        <br />
        We'll be honest about what's in it and what isn't.
        <br />
        Your progress will be waiting for you.
      </blockquote>

      <div className="cta-row">
        <a className="btn primary" href="#/faq">
          ▸ QUESTIONS? READ THE FAQ
        </a>
        <a className="btn" href="#/galaxy">
          SEE THE GALAXY
        </a>
      </div>
    </main>
  );
}
