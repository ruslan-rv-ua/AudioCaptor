import type { Theme } from "../types";

const STORAGE_KEY = "audiocaptor_theme";

let preference = $state<Theme>("auto");
let mediaQuery: MediaQueryList | null = null;

function resolve(pref: Theme): "light" | "dark" {
  if (pref === "auto") {
    return window.matchMedia("(prefers-color-scheme: dark)").matches
      ? "dark"
      : "light";
  }
  return pref;
}

function applyToDOM(t: "light" | "dark") {
  document.documentElement.setAttribute("data-theme", t);
}

function onMediaChange(e: MediaQueryListEvent) {
  if (preference === "auto") applyToDOM(e.matches ? "dark" : "light");
}

export function getTheme() {
  return {
    get preference() { return preference; },
  };
}

/** Call once in App.svelte onMount, after settings are loaded. */
export function initTheme(saved: Theme) {
  if (mediaQuery) mediaQuery.removeEventListener("change", onMediaChange);
  preference = saved;
  localStorage.setItem(STORAGE_KEY, preference);
  mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
  applyToDOM(preference === "auto" ? (mediaQuery.matches ? "dark" : "light") : preference);
  mediaQuery.addEventListener("change", onMediaChange);
}

/** Call when the user changes the theme in Settings. */
export function setTheme(t: Theme) {
  preference = t;
  localStorage.setItem(STORAGE_KEY, t);
  applyToDOM(resolve(t));
}

/** Call in App.svelte onMount cleanup. */
export function cleanupTheme() {
  mediaQuery?.removeEventListener("change", onMediaChange);
}
