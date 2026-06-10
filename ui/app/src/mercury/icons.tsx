// Inline SVG icon set (no icon font), ported from the design handoff.

interface IconProps {
  size?: number;
  color?: string;
  className?: string;
}

function svgProps(size: number, color: string, strokeWidth: number) {
  return {
    width: size,
    height: size,
    viewBox: "0 0 24 24",
    fill: "none",
    stroke: color,
    strokeWidth,
    strokeLinecap: "round" as const,
    strokeLinejoin: "round" as const,
    "aria-hidden": true,
  };
}

/** Mercury's identity mark -- used in the inspector + identity surfaces. */
export function Helix({ size = 14, color = "currentColor" }: IconProps) {
  return (
    <svg {...svgProps(size, color, 1.6)} style={{ flexShrink: 0 }}>
      <path d="M5 3c0 4 14 4 14 9s-14 5-14 9" />
      <path d="M19 3c0 4-14 4-14 9s14 5 14 9" />
      <path d="M7 6h10" />
      <path d="M7 18h10" />
      <path d="M9 9h6" />
      <path d="M9 15h6" />
    </svg>
  );
}

export function GearIcon({ size = 13, color = "currentColor" }: IconProps) {
  return (
    <svg {...svgProps(size, color, 1.7)} style={{ flexShrink: 0 }}>
      <circle cx="12" cy="12" r="3" />
      <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09a1.65 1.65 0 0 0-1-1.51 1.65 1.65 0 0 0-1.82.33l-.06.06A2 2 0 1 1 4.27 16.96l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09a1.65 1.65 0 0 0 1.51-1 1.65 1.65 0 0 0-.33-1.82l-.06-.06A2 2 0 1 1 7.04 4.27l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
    </svg>
  );
}

export function RailIcon({ size = 12, color = "currentColor" }: IconProps) {
  return (
    <svg {...svgProps(size, color, 1.7)} style={{ flexShrink: 0 }}>
      <rect x="3" y="4" width="18" height="16" rx="2" />
      <path d="M9 4v16" />
    </svg>
  );
}

export function SearchIcon({ size = 12, color = "currentColor" }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke={color} strokeWidth={2} aria-hidden>
      <circle cx="11" cy="11" r="7" />
      <path d="m20 20-3.5-3.5" />
    </svg>
  );
}

export function SendIcon({ size = 15, color = "currentColor", className }: IconProps) {
  return (
    <svg {...svgProps(size, color, 2.4)} className={className}>
      <path d="M12 19V5" />
      <path d="m5 12 7-7 7 7" />
    </svg>
  );
}

export function PaperclipIcon({ size = 15, color = "currentColor", className }: IconProps) {
  return (
    <svg {...svgProps(size, color, 2)} className={className}>
      <path d="m21.4 11.6-8.5 8.5a6 6 0 0 1-8.5-8.5l9.2-9.2a4 4 0 1 1 5.7 5.7l-9.2 9.2a2 2 0 1 1-2.8-2.8l8.5-8.5" />
    </svg>
  );
}

/** Small padlock -- composer blocked state. */
export function LockIcon({ size = 14, color = "currentColor", className }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke={color} strokeWidth={2.2} strokeLinecap="round" className={className} aria-hidden>
      <rect x="5" y="11" width="14" height="9" rx="2" />
      <path d="M8 11V8a4 4 0 0 1 8 0v3" />
    </svg>
  );
}

/** Larger padlock -- bootstrap lock hero. */
export function LockBigIcon({ size = 24, color = "currentColor", className }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke={color} strokeWidth={2} className={className} aria-hidden>
      <rect x="4" y="10" width="16" height="11" rx="2" />
      <path d="M8 10V7a4 4 0 0 1 8 0v3" />
    </svg>
  );
}

export function ShieldIcon({ size = 13, color = "currentColor", className }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke={color} strokeWidth={2} className={className} aria-hidden>
      <path d="M12 2 4 5v6c0 5 3.5 9 8 11 4.5-2 8-6 8-11V5l-8-3Z" />
    </svg>
  );
}

export function ShieldCheckIcon({ size = 18, color = "currentColor", className }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke={color} strokeWidth={2} className={className} aria-hidden>
      <path d="M12 2 4 5v6c0 5 3.5 9 8 11 4.5-2 8-6 8-11V5l-8-3Z" />
      <path d="M9 12l2 2 4-4" />
    </svg>
  );
}

export function XCircleIcon({ size = 14, color = "currentColor", className }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke={color} strokeWidth={2.2} className={className} aria-hidden>
      <circle cx="12" cy="12" r="10" />
      <path d="M4.9 4.9 19.1 19.1" />
    </svg>
  );
}

export function RefreshIcon({ size = 22, color = "currentColor", className }: IconProps) {
  return (
    <svg {...svgProps(size, color, 1.6)} className={className}>
      <path d="M21 12a8 8 0 1 1-3.13-6.31" />
      <path d="M21 4v6h-6" />
    </svg>
  );
}

/** Spinning loader -- CSS-driven ring. */
export function Spinner({ color = "var(--mc-warn)" }: { color?: string }) {
  return (
    <span
      style={{
        width: 12,
        height: 12,
        borderRadius: "50%",
        border: `1.5px solid ${color}33`,
        borderTopColor: color,
        animation: "mc-spin 0.9s linear infinite",
        flexShrink: 0,
        marginTop: 2,
      }}
    />
  );
}
