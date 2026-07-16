export type ToastTone = "success" | "error" | "info";

export interface ToastMessage {
  id: string;
  title: string;
  description?: string;
  tone: ToastTone;
}

export const TOAST_EVENT = "silk:toast";
const FLASH_TOAST_KEY = "silk:flash-toasts";

function createId() {
  return globalThis.crypto?.randomUUID?.() ?? `${Date.now()}-${Math.random()}`;
}

function message(tone: ToastTone, title: string, description?: string): ToastMessage {
  return { id: createId(), title, description, tone };
}

function show(tone: ToastTone, title: string, description?: string) {
  if (typeof window === "undefined") return;
  window.dispatchEvent(
    new CustomEvent<ToastMessage>(TOAST_EVENT, { detail: message(tone, title, description) }),
  );
}

function flash(tone: ToastTone, title: string, description?: string) {
  if (typeof window === "undefined") return;
  const queued = consumeFlashToasts();
  window.sessionStorage.setItem(
    FLASH_TOAST_KEY,
    JSON.stringify([...queued, message(tone, title, description)]),
  );
}

export function consumeFlashToasts(): ToastMessage[] {
  if (typeof window === "undefined") return [];
  const stored = window.sessionStorage.getItem(FLASH_TOAST_KEY);
  window.sessionStorage.removeItem(FLASH_TOAST_KEY);
  if (!stored) return [];
  try {
    const parsed: unknown = JSON.parse(stored);
    return Array.isArray(parsed) ? (parsed as ToastMessage[]) : [];
  } catch {
    return [];
  }
}

export const toast = {
  success: (title: string, description?: string) => show("success", title, description),
  error: (title: string, description?: string) => show("error", title, description),
  info: (title: string, description?: string) => show("info", title, description),
  flash: {
    success: (title: string, description?: string) => flash("success", title, description),
    error: (title: string, description?: string) => flash("error", title, description),
    info: (title: string, description?: string) => flash("info", title, description),
  },
};
