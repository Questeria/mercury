import { createContext } from "react";

/** When true (set by the live app for not-yet-wired screens), PanelShell shows a "preview" banner so a
 *  mockup panel can never be mistaken for a working, backend-enforced feature. Demo usage leaves it false. */
export const PreviewContext = createContext(false);

/** When true (provided by the unified Settings hub), PanelShell renders its content INLINE — no
 *  backdrop, no centered modal chrome, no close button — so a panel can live inside the hub's
 *  content pane. Default false everywhere else, so standalone modal usage is byte-for-byte unchanged. */
export const EmbeddedContext = createContext(false);
