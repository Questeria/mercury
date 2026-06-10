// Timestamp + day formatting (ported from the design-handoff helpers).

const pad = (n: number): string => String(n).padStart(2, "0");

export function fmtClk(ts: number): string {
  const d = new Date(ts);
  return `${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

export function fmtClkSec(ts: number): string {
  const d = new Date(ts);
  return `${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`;
}

/** Whether two timestamps fall on the same calendar day (local time). */
export function sameDay(a?: number, b?: number): boolean {
  if (!a || !b) return false;
  const x = new Date(a);
  const y = new Date(b);
  return (
    x.getFullYear() === y.getFullYear() &&
    x.getMonth() === y.getMonth() &&
    x.getDate() === y.getDate()
  );
}

/** "Today" / "Yesterday" / "Mar 15" / "Mar 15 2024". */
export function fmtDay(ts: number): string {
  const d = new Date(ts);
  const now = new Date();
  const yesterday = new Date(now);
  yesterday.setDate(yesterday.getDate() - 1);
  if (sameDay(d.getTime(), now.getTime())) return "Today";
  if (sameDay(d.getTime(), yesterday.getTime())) return "Yesterday";
  const sameYear = d.getFullYear() === now.getFullYear();
  const m = d.toLocaleString(undefined, { month: "short" });
  return sameYear ? `${m} ${d.getDate()}` : `${m} ${d.getDate()} ${d.getFullYear()}`;
}

/** Per-message timestamp; carries a date prefix when not from today. */
export function fmtMsgTime(ts: number): string {
  if (sameDay(ts, Date.now())) return fmtClkSec(ts);
  return `${fmtDay(ts)} · ${fmtClk(ts)}`;
}
