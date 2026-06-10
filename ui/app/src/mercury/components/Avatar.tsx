import type { CSSProperties } from "react";

import type { Person } from "../types";
import styles from "./Avatar.module.css";

interface AvatarProps {
  person: Person;
  size?: number;
  onClick?: () => void;
}

// Black or white initials, whichever is legible on an arbitrary background colour.
function contrastInk(hex: string): string {
  const m = hex.replace("#", "");
  if (m.length < 6) return "#ffffff";
  const r = parseInt(m.slice(0, 2), 16);
  const g = parseInt(m.slice(2, 4), 16);
  const b = parseInt(m.slice(4, 6), 16);
  const lum = (0.299 * r + 0.587 * g + 0.114 * b) / 255;
  return lum > 0.62 ? "#0b0e16" : "#ffffff";
}

/** Circular participant avatar. Precedence: a profile picture, then an exact custom
 *  colour, then (AI) the iridescent ring, else the per-author hue tint. Avatars are
 *  profile triggers. */
export function Avatar({ person, size = 28, onClick }: AvatarProps) {
  const isAi = !!person.isAi;
  const hasPhoto = !!person.avatar;
  const customColor = !hasPhoto && !isAi ? person.color : undefined;
  const style = {
    width: size,
    height: size,
    fontSize: Math.max(9, Math.round(size * 0.36)),
    ...(isAi || customColor ? {} : { ["--hue" as string]: String(person.hue) }),
    ...(hasPhoto
      ? { backgroundImage: `url(${person.avatar})`, backgroundSize: "cover", backgroundPosition: "center" }
      : {}),
    ...(customColor
      ? { backgroundColor: customColor, borderColor: customColor, color: contrastInk(customColor) }
      : {}),
  } as CSSProperties;

  const className = `${styles.avatar} ${isAi && !hasPhoto && !customColor ? styles.ai : styles.tinted}`;
  const inner = hasPhoto ? null : isAi && !customColor ? (
    <>
      <span className="iris-ring" style={{ borderRadius: "50%" }} />
      <span className="iris-text" style={{ position: "relative" }}>
        {person.short}
      </span>
    </>
  ) : (
    person.short
  );

  if (onClick) {
    return (
      <button type="button" className={className} style={style} onClick={onClick} data-tip={`Open ${person.name}`}>
        {inner}
      </button>
    );
  }
  return (
    <div className={className} style={style}>
      {inner}
    </div>
  );
}
