// A QR code rendered as a crisp SVG (no <canvas>, no innerHTML) from a tiny, dependency-free
// encoder. It encodes only a PUBLIC invite link — never key material or plaintext. Always drawn
// dark-on-white (required for scanners) regardless of app theme.
import qrcode from "qrcode-generator";

export function QrCode({ value, size = 196 }: { value: string; size?: number }) {
  const qr = qrcode(0, "M"); // auto version, medium (15%) error correction
  qr.addData(value);
  qr.make();
  const n = qr.getModuleCount();
  const margin = 2; // quiet zone (modules)
  const dim = n + margin * 2;

  let path = "";
  for (let row = 0; row < n; row++) {
    for (let col = 0; col < n; col++) {
      if (qr.isDark(row, col)) path += `M${col + margin} ${row + margin}h1v1h-1z`;
    }
  }

  return (
    <svg
      width={size}
      height={size}
      viewBox={`0 0 ${dim} ${dim}`}
      role="img"
      aria-label="Invite QR code"
      shapeRendering="crispEdges"
    >
      <rect width={dim} height={dim} fill="#ffffff" />
      <path d={path} fill="#000000" />
    </svg>
  );
}
