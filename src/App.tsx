import { Show, createSignal, onCleanup, onMount } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Toaster, toast } from "solid-sonner";
import "solid-sonner/styles.css";
import type { OverlayState, SonnerPosition, Toast as OverlayToast } from "./types";
import "./App.css";

const announcedIds = new Set<string>();
const activeIds = new Set<string>();
let stageRef: HTMLDivElement | undefined;
let lastMeasuredHeight = 0;

export default function App() {
  const [state, setState] = createSignal<OverlayState>();

  onMount(() => {
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
    requestResize();
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
      onDismiss: () => {
        activeIds.delete(item.id);
        announcedIds.delete(item.id);
        void invoke("dismiss_toast", { id: item.id });
        requestResize();
      },
    });
    requestResize();
  }

  function requestResize() {
    window.requestAnimationFrame(() => {
      measure();
      window.setTimeout(measure, 120);
      window.setTimeout(measure, 320);
    });
  }

  function measure() {
    if (!stageRef) return;
    const nodes = [
      ...stageRef.querySelectorAll<HTMLElement>("[data-sonner-toaster]"),
      ...stageRef.querySelectorAll<HTMLElement>("[data-sonner-toast]"),
    ];
    if (nodes.length === 0) return;
    const stageTop = stageRef.getBoundingClientRect().top;
    const bottoms = nodes.map((node) => node.getBoundingClientRect().bottom - stageTop);
    const tops = nodes.map((node) => node.getBoundingClientRect().top - stageTop);
    const height = Math.ceil(Math.max(...bottoms) - Math.min(...tops) + 48);
    if (!Number.isFinite(height) || height <= 0) return;
    if (Math.abs(height - lastMeasuredHeight) < 8) return;
    lastMeasuredHeight = height;
    void invoke("resize_toast_overlay", { contentHeight: height });
  }

  const position = (): SonnerPosition => state()?.sonnerPosition ?? "top-right";

  return (
    <div
      ref={(element) => {
        stageRef = element;
      }}
      class="overlay"
      classList={{ debug: state()?.settings.debugOverlay === true }}
    >
      <Show when={state()?.settings.debugOverlay}>
        <div class="debug-label">ToastDesk debug bounds</div>
      </Show>
      <Toaster
        id="overlay"
        position={position()}
        richColors
        closeButton
        expand={false}
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
