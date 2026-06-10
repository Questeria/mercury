import logoUrl from "../assets/mercury-logo.png";

export function MercuryLogo({ size = 24, className }: { size?: number; className?: string }) {
  return (
    <img
      src={logoUrl}
      width={size}
      height={size}
      alt=""
      aria-hidden
      className={className}
      draggable={false}
    />
  );
}
