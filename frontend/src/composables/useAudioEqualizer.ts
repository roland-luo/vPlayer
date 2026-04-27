import { ref, reactive } from "vue";

// --- module-level singleton state ---
// Shared across mount/unmount cycles so the AudioContext persists
// when the popup is closed and reopened.
let context: AudioContext | null = null;
let source: MediaElementAudioSourceNode | null = null;
let filters: BiquadFilterNode[] = [];
let _isConnected = false;

export const BAND_CONFIG = [
  { frequency: 60, label: "60Hz" },
  { frequency: 150, label: "150Hz" },
  { frequency: 400, label: "400Hz" },
  { frequency: 1000, label: "1kHz" },
  { frequency: 2400, label: "2.4kHz" },
  { frequency: 6000, label: "6kHz" },
  { frequency: 10000, label: "10kHz" },
  { frequency: 16000, label: "16kHz" },
] as const;

export interface Preset {
  name: string;
  label: string;
  gains: number[];
}

export const PRESETS: Preset[] = [
  { name: "flat", label: "Flat", gains: [0, 0, 0, 0, 0, 0, 0, 0] },
  { name: "pop", label: "Pop", gains: [2, 3, 1, -1, 0, 2, 3, 3] },
  { name: "rock", label: "Rock", gains: [4, 3, 1, -2, -1, 2, 3, 3] },
  { name: "jazz", label: "Jazz", gains: [3, 2, 1, 0, 0, 1, 2, 3] },
  { name: "classical", label: "Classical", gains: [3, 2, 0, -1, -1, 0, 2, 3] },
];

/** Clamp gain to [-12, 12] dB range */
function clampGain(v: number): number {
  return Math.max(-12, Math.min(12, v));
}

export function useAudioEqualizer() {
  const bands = reactive(
    BAND_CONFIG.map((c, i) => ({
      index: i,
      frequency: c.frequency,
      label: c.label,
      gain: 0,
    })),
  );

  const activePreset = ref<string | null>("flat");
  const isConnected = ref(false);

  /** Connect the equalizer into the <video> element's audio graph. */
  async function connect(): Promise<void> {
    if (_isConnected) {
      // Already wired — just sync the reactive flag.
      isConnected.value = true;
      return;
    }

    const videoEl = document.querySelector<HTMLVideoElement>("video");
    if (!videoEl) throw new Error("No video element found");

    context = new AudioContext();
    if (context.state === "suspended") {
      await context.resume();
    }

    source = context.createMediaElementSource(videoEl);

    filters = BAND_CONFIG.map((band) => {
      const f = context!.createBiquadFilter();
      f.type = "peaking";
      f.frequency.value = band.frequency;
      f.Q.value = 0.707; // Butterworth Q for smooth response
      f.gain.value = 0;
      return f;
    });

    // Chain: source → filter[0] → filter[1] → … → destination
    source.connect(filters[0]);
    for (let i = 0; i < filters.length - 1; i++) {
      filters[i].connect(filters[i + 1]);
    }
    filters[filters.length - 1].connect(context.destination);

    // Apply cached gains after reconnecting.
    bands.forEach((band) => {
      if (filters[band.index]) {
        filters[band.index].gain.value = band.gain;
      }
    });

    _isConnected = true;
    isConnected.value = true;
  }

  /** Tear down the audio graph and close the context. */
  function disconnect(): void {
    if (!_isConnected) return;

    try {
      if (source && filters.length > 0) {
        source.disconnect(filters[0]);
        filters.forEach((f) => f.disconnect());
      }
    } catch {
      // ignore teardown errors
    }

    if (context) {
      context.close().catch(() => {});
    }

    context = null;
    source = null;
    filters = [];
    _isConnected = false;
    isConnected.value = false;
  }

  /** Set the gain (dB) for a single frequency band. */
  function setGain(bandIndex: number, gain: number): void {
    const clamped = clampGain(gain);
    if (filters[bandIndex]) {
      filters[bandIndex].gain.value = clamped;
    }
    if (bands[bandIndex]) {
      bands[bandIndex].gain = clamped;
    }
    activePreset.value = null;
  }

  /** Apply a named preset to all bands. */
  function applyPreset(name: string): void {
    const preset = PRESETS.find((p) => p.name === name);
    if (!preset) return;

    preset.gains.forEach((gain, i) => {
      if (filters[i]) filters[i].gain.value = gain;
      if (bands[i]) bands[i].gain = gain;
    });
    activePreset.value = name;
  }

  /** Reset all bands to 0 dB (flat). */
  function reset(): void {
    bands.forEach((band) => {
      if (filters[band.index]) filters[band.index].gain.value = 0;
      band.gain = 0;
    });
    activePreset.value = "flat";
  }

  return {
    bands,
    activePreset,
    isConnected,
    connect,
    disconnect,
    setGain,
    applyPreset,
    reset,
  };
}
