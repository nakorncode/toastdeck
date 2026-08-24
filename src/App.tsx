import { createSignal, onCleanup, onMount } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Toaster, toast } from "solid-sonner";
import "solid-sonner/styles.css";
import type { OverlayState, SonnerPosition, Toast as OverlayToast } from "./types";
import "./App.css";

const announcedIds = new Set<string>();
const activeIds = new Set<string>();
const openingIds = new Set<string>();
const OVERLAY_PAD = 12;
let lastReportedHeight: number | null = null;

function measuredOverlayHeight(): number | null {
  const cards = [...document.querySelectorAll("[data-sonner-toast]")];
  if (cards.length === 0) return null;
  let top = Infinity;
  let bottom = -Infinity;
  for (const card of cards) {
    const rect = card.getBoundingClientRect();
    top = Math.min(top, rect.top);
    bottom = Math.max(bottom, rect.bottom);
  }
  return Math.ceil(bottom - top + OVERLAY_PAD * 2);
}

function reportOverlayHeight() {
  if (isHarness()) return;
  const height = measuredOverlayHeight();
  if (height == null) {
    lastReportedHeight = null;
    return;
  }
  if (height === lastReportedHeight) return;
  lastReportedHeight = height;
  void invoke("report_overlay_height", { height }).catch(() => {});
}

function scheduleOverlayHeightReport() {
  requestAnimationFrame(() => {
    requestAnimationFrame(reportOverlayHeight);
  });
}

const HARNESS_SETTINGS: OverlayState["settings"] = {
  launchOnStartup: false,
  soundEnabled: false,
  soundPreset: "aosp-argon",
  overlayPlacement: "topRight",
  cardDuration: "infinite",
  debugOverlay: true,
  windowsCapture: false,
  showLaunchToast: false,
};

const HARNESS_STACK: OverlayState = {
  settings: HARNESS_SETTINGS,
  toasts: [
    { id: "t1", title: "Test 1", body: "First card body for layout.", kind: "test" },
    { id: "t2", title: "Test 2", body: "Second card body for layout.", kind: "test" },
  ],
  sonnerPosition: "top-right",
  durationMs: null,
};

const HARNESS_SHORT: OverlayState = {
  settings: HARNESS_SETTINGS,
  toasts: [{ id: "short", title: "Test 1", body: "First card body for layout.", kind: "test" }],
  sonnerPosition: "top-right",
  durationMs: null,
};

const HARNESS_LONG: OverlayState = {
  settings: HARNESS_SETTINGS,
  toasts: [
    {
      id: "long-4",
      title: "Calendar",
      body: "Standup with design\nBring the Q3 deck\nRoom 4B at 10:30",
      kind: "test",
    },
  ],
  sonnerPosition: "top-right",
  durationMs: null,
};

function harnessState(): OverlayState | undefined {
  const harness = new URLSearchParams(window.location.search).get("harness");
  if (harness === "long") return HARNESS_LONG;
  if (harness === "short") return HARNESS_SHORT;
  if (harness === "1" || harness === "stack") return HARNESS_STACK;
  return undefined;
}

function isHarness() {
  return harnessState() !== undefined;
}

export default function App() {
  const [state, setState] = createSignal<OverlayState>();
  let overlayEl: HTMLDivElement | undefined;

  onMount(() => {
    const harness = harnessState();
    if (harness) {
      setState(harness);
      window.setTimeout(() => syncToasts(harness), 50);
      return;
    }

    let unlisten: (() => void) | undefined;
    void listen<OverlayState>("toastdesk://state", (event) => {
      setState(event.payload);
      syncToasts(event.payload);
    })
      .then(async (fn) => {
        unlisten = fn;
        await refreshState();
        await invoke("mark_overlay_ready");
      })
      .catch(() => {
        void invoke("mark_overlay_ready").catch(() => {});
      });
    const observer = new MutationObserver(() => scheduleOverlayHeightReport());
    if (overlayEl) {
      observer.observe(overlayEl, { childList: true, subtree: true, characterData: true });
    }
    onCleanup(() => {
      unlisten?.();
      observer.disconnect();
    });
  });

  async function refreshState() {
    const snapshot = await invoke<OverlayState>("get_overlay_state");
    setState(snapshot);
    syncToasts(snapshot);
  }

  function syncToasts(snapshot: OverlayState) {
    const visibleIds = new Set(snapshot.toasts.map((item) => item.id));
    for (const item of snapshot.toasts) {
      showToast(item, snapshot.durationMs);
    }
    for (const id of Array.from(activeIds)) {
      if (!visibleIds.has(id)) {
        activeIds.delete(id);
        announcedIds.delete(id);
        toast.dismiss(id);
      }
    }
    scheduleOverlayHeightReport();
  }

  function showToast(item: OverlayToast, durationMs: number | null) {
    if (announcedIds.has(item.id)) return;
    announcedIds.add(item.id);
    activeIds.add(item.id);
    toast.info(item.title, {
      id: item.id,
      toasterId: "overlay",
      description: item.body,
      duration: durationMs ?? Number.POSITIVE_INFINITY,
      closeButton: true,
      testId: item.id,
      onDismiss: () => {
        activeIds.delete(item.id);
        announcedIds.delete(item.id);
        const opened = openingIds.has(item.id);
        openingIds.delete(item.id);
        if (!isHarness() && !opened) {
          void invoke("dismiss_toast", { id: item.id });
        }
      },
    });
  }

  function handleOverlayClick(event: MouseEvent) {
    const target = event.target as HTMLElement | null;
    if (!target || target.closest("button") || isHarness()) {
      return;
    }
    const card = target.closest("[data-sonner-toast]");
    if (!(card instanceof HTMLElement)) {
      return;
    }
    const id = card.getAttribute("data-testid");
    if (!id || id === "debug-sample" || id === "capture-denied" || id === "launch") {
      return;
    }
    openingIds.add(id);
    void invoke("open_toast", { id });
  }

  const position = (): SonnerPosition => state()?.sonnerPosition ?? "top-right";
  const fromBottom = () => position().startsWith("bottom");

  return (
    <div
      ref={(el) => {
        overlayEl = el;
      }}
      class="overlay"
      classList={{
        debug: state()?.settings.debugOverlay === true,
        "from-bottom": fromBottom(),
      }}
      onClick={handleOverlayClick}
    >
      <Toaster
        id="overlay"
        position={position()}
        richColors
        closeButton
        expand
        gap={12}
        offset={12}
        visibleToasts={4}
        duration={state()?.durationMs ?? Number.POSITIVE_INFINITY}
        pauseWhenPageIsHidden={false}
        toastOptions={{
          closeButtonAriaLabel: "Dismiss",
          classNames: {
            toast: "overlay-toast",
            title: "overlay-toast-title",
            description: "overlay-toast-body",
          },
        }}
      />
    </div>
  );
}
