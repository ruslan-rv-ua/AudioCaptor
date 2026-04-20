import { setLocale } from "../paraglide/runtime";

/**
 * Initialize the app language. Call EXACTLY ONCE at startup.
 */
export function initLanguage(lang: "en" | "uk") {
  setLocale(lang, { reload: false });
  document.documentElement.lang = lang;
}
