import { mkdirSync } from "node:fs";
import { spawn } from "node:child_process";
import { chromium } from "playwright";

const PORT = process.env.OVERLAY_HARNESS_PORT ?? "4173";
const BASE = process.env.OVERLAY_HARNESS_URL ?? `http://127.0.0.1:${PORT}/`;
const PAD = 12;
const CARD = 74;
const VIEWPORT = {
  width: 380,
  height: PAD * 2 + CARD,
};
const VARIANTS = ["long", "short"];

function startPreview() {
  if (process.env.OVERLAY_HARNESS_URL) {
    return { stop() {} };
  }
  const child = spawn(
    process.execPath,
    ["./node_modules/vite/bin/vite.js", "preview", "--host", "127.0.0.1", "--port", PORT, "--strictPort"],
    { stdio: "pipe" },
  );
  return {
    stop() {
      child.kill();
    },
  };
}

async function waitForServer(url, timeoutMs) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    try {
      const response = await fetch(url, { redirect: "manual" });
      if (response.ok || response.status === 404) return;
    } catch {
      // preview not up yet
    }
    await new Promise((resolve) => setTimeout(resolve, 150));
  }
  throw new Error(`preview did not start: ${url}`);
}

function uniqueLineCount(lines) {
  return new Set(lines.map((line) => line.top.toFixed(1))).size;
}

async function measure(page) {
  return page.evaluate(() => {
    const overlay = document.querySelector(".overlay");
    const toast = document.querySelector("[data-sonner-toast]");
    const title = toast?.querySelector("[data-title]");
    const description = toast?.querySelector("[data-description]");
    const overlayRect = overlay?.getBoundingClientRect();
    const toastRect = toast?.getBoundingClientRect();

    function lineRects(node) {
      if (!node || !node.firstChild) return [];
      const range = document.createRange();
      range.selectNodeContents(node);
      return [...range.getClientRects()]
        .filter((rect) => rect.height > 1)
        .map((rect) => ({
          top: rect.top,
          bottom: rect.bottom,
        }));
    }

    const overlayBottom = overlayRect?.bottom ?? 0;
    const descriptionBox = description?.getBoundingClientRect();
    const titleLines = lineRects(title);
    const bodyLines = lineRects(description).filter((line) =>
      descriptionBox ? line.top < descriptionBox.bottom - 0.5 : true,
    );
    const lines = [...titleLines, ...bodyLines];
    const clippedLines = lines.filter((line) => line.bottom > overlayBottom + 1);
    const descriptionStyle = description ? getComputedStyle(description) : null;

    return {
      toastCount: document.querySelectorAll("[data-sonner-toast]").length,
      lines,
      clippedLineCount: clippedLines.length,
      descriptionClipped: description
        ? description.scrollHeight - description.clientHeight > 2
        : false,
      toastClipped: toast ? toast.scrollHeight - toast.clientHeight > 2 : false,
      overflow: toastRect && overlayRect ? toastRect.bottom - overlayRect.bottom : 0,
      toastHeight: toastRect?.height ?? 0,
      overlayHeight: overlayRect?.height ?? 0,
      viewport: { width: window.innerWidth, height: window.innerHeight },
      whiteSpace: descriptionStyle?.whiteSpace ?? null,
      text: `${title?.textContent ?? ""}\n${description?.textContent ?? ""}`,
    };
  });
}

function judge(variant, snapshot) {
  const failures = [];
  const lineCount = uniqueLineCount(snapshot.lines);
  if (snapshot.toastCount !== 1) {
    failures.push(`expected 1 toast, got ${snapshot.toastCount}`);
  }
  if (variant === "long" && lineCount > 3) {
    failures.push(`expected at most 3 visible text lines, got ${lineCount}`);
  }
  if (variant === "long" && lineCount < 3) {
    failures.push(`expected 3 visible text lines for the long fixture, got ${lineCount}`);
  }
  if (variant === "long" && snapshot.hugHeight <= VIEWPORT.height) {
    failures.push(
      `3-line toast should need a taller window than ${VIEWPORT.height}px, got hug ${snapshot.hugHeight}`,
    );
  }
  if (snapshot.clippedLineCount > 0) {
    failures.push(
      `${snapshot.clippedLineCount} text line(s) clipped by the overlay (cut in half)`,
    );
  }
  if (variant !== "long" && (snapshot.descriptionClipped || snapshot.toastClipped)) {
    failures.push("toast text is internally clipped (scrollHeight > clientHeight)");
  }
  if (snapshot.overflow > 2) {
    failures.push(`toast overflows overlay by ${snapshot.overflow.toFixed(1)}px`);
  }
  return { failures, lineCount };
}

const preview = startPreview();
try {
  await waitForServer(`http://127.0.0.1:${PORT}/`, 15_000);
  mkdirSync("artifacts", { recursive: true });
  const browser = await chromium.launch({ headless: true });
  const results = [];

  for (const variant of VARIANTS.map((item) => item.trim()).filter(Boolean)) {
    const startViewport =
      variant === "long" ? { width: VIEWPORT.width, height: 220 } : VIEWPORT;
    const page = await browser.newPage({ viewport: startViewport });
    await page.goto(new URL(`?harness=${variant}`, BASE).href, { waitUntil: "networkidle" });
    await page.waitForSelector("[data-sonner-toast]", { timeout: 5000 });
    await page.waitForTimeout(200);
    let snapshot = await measure(page);
    if (variant === "long") {
      const hugHeight = Math.ceil(snapshot.toastHeight + PAD * 2);
      await page.setViewportSize({ width: VIEWPORT.width, height: hugHeight });
      await page.waitForTimeout(100);
      snapshot = await measure(page);
      snapshot.hugHeight = hugHeight;
    }
    await page.screenshot({ path: `artifacts/overlay-long-${variant}.png` });
    await page.close();
    const { failures, lineCount } = judge(variant, snapshot);
    results.push({
      variant,
      ok: failures.length === 0,
      failures,
      lineCount,
      snapshot,
    });
  }

  await browser.close();
  const report = { ok: results.every((item) => item.ok), viewport: VIEWPORT, results };
  console.log(JSON.stringify(report, null, 2));
  process.exit(report.ok ? 0 : 1);
} finally {
  preview.stop();
}
