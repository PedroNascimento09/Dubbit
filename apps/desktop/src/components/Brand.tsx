interface BrandProps {
  version: string;
  demo: boolean;
}

export function Brand({ version, demo }: BrandProps) {
  return (
    <header className="brand">
      <svg className="brand-mark" viewBox="0 0 64 64" role="img" aria-label="Dubbit">
        <rect width="64" height="64" rx="16" className="brand-mark-bg" />
        <rect x="14" y="23" width="6" height="18" rx="3" className="brand-mark-bar" />
        <rect x="25" y="15" width="6" height="34" rx="3" className="brand-mark-bar" />
        <rect
          x="36"
          y="9"
          width="6"
          height="46"
          rx="3"
          className="brand-mark-bar brand-mark-bar-light"
        />
        <rect x="47" y="20" width="6" height="24" rx="3" className="brand-mark-bar" />
      </svg>
      <div className="brand-text">
        <span className="brand-name">Dubbit</span>
        <span className="brand-version">
          {demo ? "Interface preview" : `Version ${version}`}
        </span>
      </div>
    </header>
  );
}
