import type { RecordingProfile } from "../types";
import * as api from "../utils/invoke";

let profiles = $state<RecordingProfile[]>([]);
let activeProfileId = $state("default");
let loading = $state(false);

export function getProfiles() {
  return {
    get list() { return profiles; },
    get activeId() { return activeProfileId; },
    set activeId(id: string) { activeProfileId = id; },
    get active(): RecordingProfile | undefined {
      return profiles.find(p => p.id === activeProfileId);
    },
    get loading() { return loading; },
  };
}

export async function loadProfiles() {
  loading = true;
  try {
    profiles = await api.cmdListProfiles();
    const active = await api.cmdGetActiveProfile();
    activeProfileId = active.id;
  } catch (e) {
    console.error("Failed to load profiles:", e);
  } finally {
    loading = false;
  }
}

export async function saveProfile(profile: RecordingProfile) {
  await api.cmdSaveProfile(profile);
  profiles = await api.cmdListProfiles();
}

export async function deleteProfile(id: string) {
  const newActiveId = await api.cmdDeleteProfile(id);
  activeProfileId = newActiveId;
  profiles = await api.cmdListProfiles();
}

export async function selectProfile(id: string) {
  await api.cmdSelectProfile(id);
  activeProfileId = id;
}
