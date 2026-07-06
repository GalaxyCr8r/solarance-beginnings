import { CONFIG } from '../config';

/**
 * Shown once, right after login: the website is a relay station, not a
 * cockpit — the game itself is a downloadable native client.
 */
export function DownloadDialog({ onOk }: { onOk: () => void }) {
  return (
    <div className="modal-backdrop" role="dialog" aria-modal="true">
      <div className="modal">
        <div className="modal-head">▸ INCOMING TRANSMISSION</div>
        <h2>Welcome aboard, pilot.</h2>
        <p>
          You're logged in — but this website is only a <em>relay station</em>:
          chat, maps, and registration. To actually fly, you need the native
          game client.
        </p>
        <p>
          <a href={CONFIG.itchUrl} target="_blank" rel="noopener">
            Download the latest client on itch.io →
          </a>
        </p>
        <p className="dim">
          <span
            className="fake-link"
            title="Steam page — someday. Not yet."
            aria-disabled="true"
          >
            Steam — coming eventually
          </span>
        </p>
        <div className="modal-actions">
          <button className="btn primary" onClick={onOk} autoFocus>
            OK
          </button>
        </div>
      </div>
    </div>
  );
}
