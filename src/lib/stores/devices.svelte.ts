import { getAudioDevices } from "../utils/invoke";
import type { AudioDevice } from "../types";

let devices = $state<AudioDevice[]>([]);
let loading = $state(false);
let error = $state<string | null>(null);

export function getDevices() {
  return {
    get all() { return devices; },
    get microphones() { return devices.filter((d) => d.isInput); },
    get loopbacks() { return devices.filter((d) => !d.isInput); },
    get loading() { return loading; },
    get error() { return error; },
  };
}

export async function refreshDevices() {
  loading = true;
  error = null;
  try {
    devices = await getAudioDevices();
  } catch (e) {
    error = e instanceof Error ? e.message : String(e);
  } finally {
    loading = false;
  }
}
