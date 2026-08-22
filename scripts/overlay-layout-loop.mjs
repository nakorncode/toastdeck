import { chromium } from "playwright";

const URL = process.env.OVERLAY_HARNESS_URL ?? "http://localhost:1420/?harness=1";
const PAD = 12;
const CARD = 74;
const GAP = 12;
const TOAST_COUNT = 2;
const VIEWPORT = {
  width: 380,
  height: PAD * 2 + TOAST_COUNT * CARD + (TOAST_COUNT - 1) * GAP,
};

function overlapArea(a, b) {
  const x = Math.max(0, Math.min(a.right, b.right) - Math.max(a.left, b.left));
  const y = Math.max(0, Math.min(a.bottom, b.bottom) - Math.max(a.top, b.top));
  return x * y;
}

async function measure(page) {
  return page.evaluate(() => {
    const overlay = document.querySelector(".overlay");
    const toasts = [...document.querySelectorAll("[data-sonner-toast]")].map((node) => {
      const rect = node.getBoundingClientRect();
      return {
        title: node.textContent?.slice(0, 40) ?? "",
        left: rect.left,
        top: rect.top,
        right: rect.right,
        bottom: rect.bottom,
        width: rect.width,
        height: rect.height,
      };
    });
    const overlayRect = overlay?.getBoundingClientRect();
    return {
      toastCount: toasts.length,
      toasts,
      overlay: overlayRect
        ? {
            width: overlayRect.width,
            height: overlayRect.height,
            top: overlayRect.top,
            left: overlayRect.left,
          }
        : null,
      viewport: { width: window.innerWidth, height: window.innerHeight },
      debug: overlay?.classList.contains("debug") ?? false,
    };
  });
}

const browser = await chromium.launch({ headless: true });
const page = await browser.newPage({ viewport: VIEWPORT });
await page.goto(URL, { waitUntil: "networkidle" });
await page.waitForSelector("[data-sonner-toast]", { timeout: 5000 });
await page.waitForTimeout(200);

const before = await measure(page);
await page.locator("[data-sonner-toast]").first().hover();
await page.waitForTimeout(400);
const afterHover = await measure(page);

const pairs = [];
for (let i = 0; i < before.toasts.length; i += 1) {
  for (let j = i + 1; j < before.toasts.length; j += 1) {
    pairs.push(overlapArea(before.toasts[i], before.toasts[j]));
  }
}
const maxOverlap = Math.max(0, ...pairs);
const heightDelta = (afterHover.overlay?.height ?? 0) - (before.overlay?.height ?? 0);
const overlayMatchesViewport =
  before.overlay &&
  Math.abs(before.overlay.width - before.viewport.width) <= 2 &&
  Math.abs(before.overlay.height - before.viewport.height) <= 2 &&
  Math.abs(before.overlay.top) <= 2 &&
  Math.abs(before.overlay.left) <= 2;

const OFFSET = PAD;
const gaps = [];
for (let i = 1; i < before.toasts.length; i += 1) {
  gaps.push(before.toasts[i].top - before.toasts[i - 1].bottom);
}
const firstToast = before.toasts[0];
const last = before.toasts[before.toasts.length - 1];
const insetTop = firstToast?.top ?? 0;
const insetRight = before.overlay ? before.overlay.width - (firstToast?.right ?? 0) : 0;
const overflow = last && before.overlay ? last.bottom - before.overlay.height : 0;
const hoverOverlap = (() => {
  const hoverPairs = [];
  for (let i = 0; i < afterHover.toasts.length; i += 1) {
    for (let j = i + 1; j < afterHover.toasts.length; j += 1) {
      hoverPairs.push(overlapArea(afterHover.toasts[i], afterHover.toasts[j]));
    }
  }
  return Math.max(0, ...hoverPairs);
})();

const failures = [];
if (before.toastCount < 2) {
  failures.push(`expected 2 stacked toasts, got ${before.toastCount}`);
}
if (maxOverlap > 8) {
  failures.push(`toasts overlap by ${maxOverlap.toFixed(1)}px²`);
}
if (hoverOverlap > 8) {
  failures.push(`toasts overlap on hover by ${hoverOverlap.toFixed(1)}px²`);
}
if (heightDelta < -2) {
  failures.push(`overlay shrank on hover by ${Math.abs(heightDelta).toFixed(1)}px`);
}
if (gaps.some((gap) => Math.abs(gap - GAP) > 2)) {
  failures.push(`stack gap ${JSON.stringify(gaps)} != ${GAP}px`);
}
if (Math.abs(insetTop - OFFSET) > 2) {
  failures.push(`top inset ${insetTop}px != ${OFFSET}px`);
}
if (Math.abs(insetRight - OFFSET) > 2) {
  failures.push(`side inset ${insetRight}px != ${OFFSET}px`);
}
const bottomInset = before.overlay && last ? before.overlay.height - last.bottom : 0;
if (overflow > 2) {
  failures.push(`toasts overflow overlay by ${overflow.toFixed(1)}px`);
}
if (Math.abs(bottomInset - OFFSET) > 3) {
  failures.push(`bottom inset ${bottomInset}px != ${OFFSET}px`);
}
if (!before.debug) {
  failures.push("debug class missing");
}
if (!overlayMatchesViewport) {
  failures.push(
    `debug overlay box ${JSON.stringify(before.overlay)} != viewport ${JSON.stringify(before.viewport)}`,
  );
}

const report = {
  ok: failures.length === 0,
  failures,
  gaps,
  insetTop,
  insetRight,
  bottomInset,
  overflow,
  before,
  afterHover: {
    overlayHeight: afterHover.overlay?.height,
    heightDelta,
    toasts: afterHover.toasts,
  },
};

console.log(JSON.stringify(report, null, 2));
await browser.close();
process.exit(report.ok ? 0 : 1);
