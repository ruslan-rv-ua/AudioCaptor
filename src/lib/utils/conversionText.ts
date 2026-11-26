import * as m from "../../paraglide/messages";
import { getLocale } from "../../paraglide/runtime";

/**
 * Time left as it is said aloud: "about 20 seconds", "about 2 minutes".
 * Seconds are rounded up to five, so the phrase does not change every second.
 */
export function formatTimeLeft(seconds: number): string {
  if (seconds < 55) {
    return m.time_about_seconds({ count: Math.max(5, Math.ceil(seconds / 5) * 5) });
  }
  const minutes = Math.round(seconds / 60);
  // The noun follows the number: "1 minute", and in Ukrainian also 21, 31, ...
  const one = new Intl.PluralRules(getLocale()).select(minutes) === "one";
  return one
    ? m.time_about_minute_one({ count: minutes })
    : m.time_about_minutes_other({ count: minutes });
}

/**
 * What Alt+I says about the conversions: that one runs, about how long all of
 * them will take, and how many files wait.
 */
export function conversionInfo(format: string, etaSeconds: number | null, waiting: number): string {
  const running =
    etaSeconds === null
      ? m.conversion_in_progress({ format })
      : m.conversion_time_left({ format, time: formatTimeLeft(etaSeconds) });
  return waiting > 0 ? `${running} ${m.conversion_files_waiting({ count: waiting })}` : running;
}
