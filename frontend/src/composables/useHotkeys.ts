import { onMounted, onUnmounted, type Ref } from "vue";

export type HotkeyOptions = {
  enabled?: Ref<boolean> | (() => boolean);
  target?: "global" | "element";
  element?: HTMLElement | null;
  preventDefault?: boolean;
  ignoreWhenTyping?: boolean;
};

function isTypingTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  if (target.isContentEditable) return true;
  const tag = target.tagName.toLowerCase();
  return tag === "input" || tag === "textarea" || tag === "select";
}

function keyMatches(event: KeyboardEvent, key: string): boolean {
  const parts = key.toLowerCase().split(/[+]/);
  const keyPart = parts[parts.length - 1];
  const modifiers = parts.slice(0, -1);

  const hasCtrl = modifiers.includes("ctrl") || modifiers.includes("control");
  const hasMeta = modifiers.includes("meta") || modifiers.includes("cmd") || modifiers.includes("command");
  const hasShift = modifiers.includes("shift");
  const hasAlt = modifiers.includes("alt");

  if (hasCtrl && !(event.ctrlKey || event.metaKey)) return false;
  if (hasMeta && !(event.metaKey || event.ctrlKey)) return false;
  if (hasShift && !event.shiftKey) return false;
  if (hasAlt && !event.altKey) return false;

  // For single-letter keys, match event.key (case-insensitive)
  if (event.key.length === 1) {
    return event.key.toLowerCase() === keyPart;
  }
  // For special keys (Enter, Escape, etc.), match event.key directly
  return event.key.toLowerCase() === keyPart;
}

export function useHotkey(
  key: string,
  handler: (event: KeyboardEvent) => void,
  options: HotkeyOptions = {}
) {
  const { enabled, target = "global", element, preventDefault = true, ignoreWhenTyping = true } = options;

  const isEnabled = (): boolean => {
    if (!enabled) return true;
    if (typeof enabled === "function") return enabled();
    return enabled.value;
  };

  const listener = (event: KeyboardEvent) => {
    if (!isEnabled()) return;
    if (ignoreWhenTyping && isTypingTarget(event.target)) return;
    if (!keyMatches(event, key)) return;

    if (preventDefault) {
      event.preventDefault();
    }
    handler(event);
  };

  onMounted(() => {
    if (target === "element" && element) {
      element.addEventListener("keydown", listener);
    } else {
      window.addEventListener("keydown", listener);
    }
  });

  onUnmounted(() => {
    if (target === "element" && element) {
      element.removeEventListener("keydown", listener);
    } else {
      window.removeEventListener("keydown", listener);
    }
  });

  return {
    isEnabled,
  };
}
