import { listen } from "@tauri-apps/api/event";
import type { ConversionFailure, ConversionStatus } from "../types";

// The queue of recordings being converted to a compressed format, as the
// backend reports it (the `conversion-status` event).
let files = $state(0);
let waiting = $state(0);
let format = $state<string | null>(null);
let progress = $state(0);
let etaSeconds = $state<number | null>(null);
// Recordings that could not be converted since the queue last started to work.
let failed = $state(0);

export function getConversion() {
  return {
    /** True while a recording is being converted or waits for it. */
    get busy() { return files > 0; },
    /** Files waiting behind the one being converted. */
    get waiting() { return waiting; },
    /** The format being written, e.g. "Opus". */
    get format() { return format; },
    /** How much of the file being converted is done, 0 to 1. */
    get progress() { return progress; },
    /** Seconds until the queue is empty; null while there is nothing to go by. */
    get etaSeconds() { return etaSeconds; },
    get failed() { return failed; },
  };
}

/** `onFailed` runs for each recording that could not be converted. */
export async function initConversionListener(onFailed: (failure: ConversionFailure) => void) {
  await listen<ConversionStatus>("conversion-status", (event) => {
    const status = event.payload;
    if (files === 0 && status.files > 0) failed = 0;
    files = status.files;
    waiting = status.waiting;
    format = status.format;
    progress = status.progress;
    etaSeconds = status.etaSeconds;
  });

  await listen<ConversionFailure>("conversion-failed", (event) => {
    failed += 1;
    onFailed(event.payload);
  });
}
