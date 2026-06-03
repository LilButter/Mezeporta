// Minimal shim for @tauri-apps/api/event
// Provides listen() and emit() for compatibility with existing Vue code.

const BRIDGE_KEY = "__MEZEPORTA_BRIDGE__";
const LEGACY_BRIDGE_KEY = "__BUTTER_BRIDGE__";

function getBridge() {
  if (typeof window === "undefined") return null;
  return window[BRIDGE_KEY] ?? window[LEGACY_BRIDGE_KEY] ?? null;
}

// Tauri: listen(event, handler) -> Promise<UnlistenFn>
// We forward Electron events from main process ("butter:event") through the bridge.
// We filter by common fields (type/name/event) so backend can send {type: "...", payload: ...}.
export async function listen(eventName, handler) {
  const bridge = getBridge();
  if (!bridge || !bridge.onEvent) {
    throw new Error("Event bridge not available");
  }

  const unsubscribe = bridge.onEvent((evt) => {
    try {
      const name = evt?.type ?? evt?.event ?? evt?.name;
      if (name === eventName) {
        handler({ event: eventName, payload: evt?.payload ?? evt });
      }
    } catch (_) {
      // ignore handler errors
    }
  });

  // Return Tauri-like unlisten function
  return async () => {
    try { if (typeof unsubscribe === "function") unsubscribe(); } catch (_) {}
  };
}

// Tauri: emit(event, payload)
// For now, forward to backend as an rpc notification-style method if available.
// If not implemented, no-op (many UIs don't rely on renderer->backend emits).
export async function emit(name, payload) {
  const bridge = getBridge();
  if (!bridge || !bridge.invoke) return;
  try {
    await bridge.invoke("event.emit", { name, payload });
  } catch (_) {
    // no-op
  }
}
