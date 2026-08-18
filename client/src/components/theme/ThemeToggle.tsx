import { MoonIcon, SunIcon } from "@phosphor-icons/react";
import { useEffect, useState } from "react";

const THEME_STORAGE_KEY = "silk-theme";
const THEME_CHANGE_EVENT = "silk-theme-change";

type Theme = "light" | "dark";

function getSystemTheme(): Theme {
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

function getStoredTheme(): Theme | null {
  try {
    const storedTheme = window.localStorage.getItem(THEME_STORAGE_KEY);
    return storedTheme === "light" || storedTheme === "dark" ? storedTheme : null;
  } catch {
    return null;
  }
}

function getCurrentTheme(): Theme {
  const documentTheme = document.documentElement.dataset.theme;
  if (documentTheme === "light" || documentTheme === "dark") return documentTheme;
  return getStoredTheme() ?? getSystemTheme();
}

function setTheme(theme: Theme) {
  document.documentElement.dataset.theme = theme;

  try {
    window.localStorage.setItem(THEME_STORAGE_KEY, theme);
  } catch {
    // Private browsing and blocked storage should not prevent the toggle working.
  }

  window.dispatchEvent(new CustomEvent<Theme>(THEME_CHANGE_EVENT, { detail: theme }));
}

interface ThemeToggleProps {
  compact?: boolean;
}

export function ThemeToggle({ compact = false }: ThemeToggleProps) {
  const [theme, setThemeState] = useState<Theme>("light");

  useEffect(() => {
    const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
    const syncTheme = () => setThemeState(getCurrentTheme());
    const syncFromToggle = (event: Event) => {
      const themeEvent = event as CustomEvent<Theme>;
      if (themeEvent.detail === "light" || themeEvent.detail === "dark") {
        setThemeState(themeEvent.detail);
      }
    };
    const syncFromSystem = () => {
      if (!getStoredTheme()) syncTheme();
    };

    syncTheme();
    window.addEventListener(THEME_CHANGE_EVENT, syncFromToggle);
    mediaQuery.addEventListener("change", syncFromSystem);

    return () => {
      window.removeEventListener(THEME_CHANGE_EVENT, syncFromToggle);
      mediaQuery.removeEventListener("change", syncFromSystem);
    };
  }, []);

  const isDark = theme === "dark";
  const nextTheme = isDark ? "light" : "dark";
  const label = `Switch to ${nextTheme} mode`;

  return (
    <button
      aria-label={label}
      aria-pressed={isDark}
      className={`theme-toggle${compact ? " theme-toggle--compact" : ""}`}
      onClick={() => setTheme(nextTheme)}
      title={label}
      type="button"
    >
      {isDark ? <SunIcon aria-hidden size={19} weight="bold" /> : <MoonIcon aria-hidden size={19} weight="bold" />}
      <span className="theme-toggle__label">{isDark ? "Light mode" : "Dark mode"}</span>
    </button>
  );
}
