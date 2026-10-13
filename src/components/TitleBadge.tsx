export function TitleBadge({ onClick }: { onClick: () => void }) {
  return (
    <button type="button" className="title-badge" onClick={onClick} aria-label="Ultimate Windows Desktop Settings. Open About and Credits.">
      <span className="badge-bezel" aria-hidden="true" />
      <span className="badge-face">
        <span className="badge-kicker">Ultimate</span>
        <span className="badge-title">
          Windows
          <br />
          Desktop
        </span>
        <span className="badge-kicker">Settings</span>
        <span className="badge-version">v2</span>
      </span>
    </button>
  );
}
