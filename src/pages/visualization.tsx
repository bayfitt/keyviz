import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import "./sticky-indicator.css";

type Modifier = "Shift" | "Ctrl" | "Alt" | "Win";
type ModifierState = "pressed" | "latched" | "locked";

export type StickyIndicator = {
  key: Modifier;
  state: ModifierState;
};

const labels: Record<Modifier, { symbol: string; name: string }> = {
  Shift: { symbol: "⇧", name: "Shift" },
  Ctrl: { symbol: "Ctrl", name: "Control" },
  Alt: { symbol: "Alt", name: "Alt" },
  Win: { symbol: "", name: "Windows" },
};

/** The backend emits the complete current state, including an empty array. */
export function Visualization() {
  const [modifiers, setModifiers] = useState<StickyIndicator[]>([]);

  useEffect(() => {
    const subscriptions = [
      listen<StickyIndicator[]>("sticky-state", ({ payload }) => setModifiers(payload)),
    ];
    return () => {
      subscriptions.forEach((subscription) => subscription.then((unlisten) => unlisten()));
    };
  }, []);

  if (modifiers.length === 0) return null;

  return (
    <div className="sticky-indicators" aria-live="polite" aria-label="Active modifier keys">
      {modifiers.map(({ key, state }) => (
        <div
          className={`sticky-indicator sticky-indicator--${state}`}
          key={key}
          aria-label={`${labels[key].name}: ${state}`}
          title={`${labels[key].name}: ${state}`}
        >
          {key === "Win" ? (
            <svg className="sticky-indicator__windows" viewBox="0 0 24 24" aria-hidden="true">
              <path fill="currentColor" d="M1 1h10v10H1zM13 1h10v10H13zM1 13h10v10H1zM13 13h10v10H13z" />
            </svg>
          ) : (
            <span className={`sticky-indicator__symbol ${key === "Shift" ? "sticky-indicator__symbol--shift" : ""}`} aria-hidden="true">{labels[key].symbol}</span>
          )}
        </div>
      ))}
    </div>
  );
}
