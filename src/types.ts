export type OverlayPlacement =
  | "topLeft"
  | "topCenter"
  | "topRight"
  | "middleLeft"
  | "center"
  | "middleRight"
  | "bottomLeft"
  | "bottomCenter"
  | "bottomRight";

export type CardDuration = "seconds10" | "seconds30" | "minute" | "infinite";

export type SonnerPosition =
  | "top-left"
  | "top-center"
  | "top-right"
  | "bottom-left"
  | "bottom-center"
  | "bottom-right";

export type AppSettings = {
  launchOnStartup: boolean;
  soundEnabled: boolean;
  soundPreset: string;
  overlayPlacement: OverlayPlacement;
  cardDuration: CardDuration;
  debugOverlay: boolean;
};

export type Toast = {
  id: string;
  title: string;
  body: string;
  kind: string;
};

export type OverlayState = {
  settings: AppSettings;
  toasts: Toast[];
  sonnerPosition: SonnerPosition;
  durationMs: number | null;
};
