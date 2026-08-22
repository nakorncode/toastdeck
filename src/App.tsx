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

const HARNESS_STATE: OverlayState = {
  settings: {
    launchOnStartup: false,
    soundEnabled: false,
    soundPreset: "aosp-argon",
    overlayPlacement: "topRight",
    cardDuration: "infinite",
    debugOverlay: true,
    windowsCapture: false,
  },
  toasts: [
    { id: "t1", title: "Test 1", body: "First card body for layout.", kind: "test" },
    { id: "t2", title: "Test 2", body: "Second card body for layout.", kind: "test" },
  ],
  sonnerPosition: "top-right",
  durationMs: null,
};

function isHarness() {
  return new URLSearchParams(window.location.search).has("harness");
}

export default function App() {
  const [state, setState] = createSignal<OverlayState>();

  onMount(() => {
    if (isHarness()) {
      setState(HARNESS_STATE);
      window.setTimeout(() => syncToasts(HARNESS_STATE), 50);
      return;
    }

    void refreshState();
    let unlisten: (() => void) | undefined;
    void listen<OverlayState>("toastdesk://state", (event) => {
      setState(event.payload);
      syncToasts(event.payload);
    }).then((fn) => {
      unlisten = fn;
    });
    onCleanup(() => unlisten?.());
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
    if (!id || id === "debug-sample" || id === "capture-denied") {
      return;
    }
    openingIds.add(id);
    void invoke("open_toast", { id });
  }

  const position = (): SonnerPosition => state()?.sonnerPosition ?? "top-right";
  const fromBottom = () => position().startsWith("bottom");

  return (
    <div
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
            description: "overlay-toast-body",
          },
        }}
      />
    </div>
  );
}
