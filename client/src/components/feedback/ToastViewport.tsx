import {
  CheckCircleIcon,
  InfoIcon,
  WarningCircleIcon,
  XIcon,
} from "@phosphor-icons/react";
import { useCallback, useEffect, useRef, useState } from "react";
import {
  consumeFlashToasts,
  TOAST_EVENT,
  type ToastMessage,
} from "@/lib/toast";

const TOAST_LIFETIME_MS = 5_000;
const MAX_VISIBLE_TOASTS = 4;

function ToastIcon({ tone }: Pick<ToastMessage, "tone">) {
  if (tone === "success") return <CheckCircleIcon aria-hidden size={20} weight="fill" />;
  if (tone === "error") return <WarningCircleIcon aria-hidden size={20} weight="fill" />;
  return <InfoIcon aria-hidden size={20} weight="fill" />;
}

export function ToastViewport() {
  const [messages, setMessages] = useState<ToastMessage[]>([]);
  const timers = useRef(new Map<string, number>());

  const dismiss = useCallback((id: string) => {
    setMessages((current) => current.filter((entry) => entry.id !== id));
    const timer = timers.current.get(id);
    if (timer) window.clearTimeout(timer);
    timers.current.delete(id);
  }, []);

  const add = useCallback(
    (next: ToastMessage) => {
      setMessages((current) => [...current.filter((entry) => entry.id !== next.id), next].slice(-MAX_VISIBLE_TOASTS));
      const timer = window.setTimeout(() => dismiss(next.id), TOAST_LIFETIME_MS);
      timers.current.set(next.id, timer);
    },
    [dismiss],
  );

  useEffect(() => {
    for (const queued of consumeFlashToasts()) add(queued);
    const receive = (event: Event) => add((event as CustomEvent<ToastMessage>).detail);
    window.addEventListener(TOAST_EVENT, receive);
    const activeTimers = timers.current;
    return () => {
      window.removeEventListener(TOAST_EVENT, receive);
      for (const timer of activeTimers.values()) window.clearTimeout(timer);
      activeTimers.clear();
    };
  }, [add]);

  return (
    <div aria-atomic="false" aria-live="polite" className="toast-viewport">
      {messages.map((entry) => (
        <article
          className={`toast toast--${entry.tone}`}
          key={entry.id}
          role={entry.tone === "error" ? "alert" : "status"}
        >
          <ToastIcon tone={entry.tone} />
          <div>
            <strong>{entry.title}</strong>
            {entry.description && <p>{entry.description}</p>}
          </div>
          <button aria-label="Dismiss notification" onClick={() => dismiss(entry.id)} type="button">
            <XIcon aria-hidden size={16} />
          </button>
        </article>
      ))}
    </div>
  );
}
