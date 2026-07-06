import { CONFIG } from '../config';

/**
 * Shown on every page: where to actually get the game (itch.io), plus a
 * deliberately disabled Steam link for the future.
 */
export function Footer() {
  return (
    <footer className="footer">
      <div className="footer-cta">
        <span className="kicker">▸ the website is not the game</span>
        <div className="footer-buttons">
          <a
            className="btn primary"
            href={CONFIG.itchUrl}
            target="_blank"
            rel="noopener"
          >
            DOWNLOAD THE CLIENT — ITCH.IO →
          </a>
          <a
            className="btn disabled"
            href="#/"
            aria-disabled="true"
            onClick={e => e.preventDefault()}
            title="Steam page — someday. Not yet."
          >
            STEAM — COMING EVENTUALLY
          </a>
        </div>
      </div>
      <div className="footer-meta">
        <a href={CONFIG.discordUrl} target="_blank" rel="noopener">
          discord
        </a>
        <span>·</span>
        <a href={CONFIG.githubUrl} target="_blank" rel="noopener">
          github
        </a>
        <span>·</span>
        <span>
          © {new Date().getFullYear()} Karl Nyborg · pre-alpha · built solo, in
          evenings
        </span>
      </div>
    </footer>
  );
}
