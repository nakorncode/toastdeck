//! The toast overlay: one layered window drawn with Direct2D and DirectWrite.
//! Layout, placement and hit testing live here so they can be tested without a window.

use windows::core::w;
use windows::Win32::Foundation::{HWND, RECT};

use crate::model::{Toast, CAPTURE_DENIED_ID, DEBUG_TOAST_ID};
use crate::settings::OverlayPlacement;

pub const OVERLAY_WIDTH: i32 = 380;
pub const PAD: i32 = 12;
pub const GAP: i32 = 12;
pub const CARD_PAD: i32 = 12;
pub const CLOSE_SIZE: i32 = 28;
const CLOSE_FONT_SIZE: f32 = 18.0;
pub const BUTTON_HEIGHT: i32 = 26;
pub const MARGIN: i32 = 16;
/// Title shows one line and the body shows at most two, as in v2.
pub const TITLE_LINES: f32 = 1.0;
pub const BODY_LINES: f32 = 2.0;
const TITLE_FONT_SIZE: f32 = 14.0;
const BODY_FONT_SIZE: f32 = 12.0;
const LINE_FACTOR: f32 = 1.4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub width: i32,
    pub height: i32,
}

impl Rect {
    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.left && x < self.left + self.width && y >= self.top && y < self.top + self.height
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hit {
    /// Click on a card body: open the source app and dismiss the card.
    Card(usize),
    /// Click on a card's close button: dismiss the card only.
    Close(usize),
    /// Click on "Dismiss all": dismiss captured notifications.
    DismissAll,
}

/// Size of the layout for one card: title height and body height in pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CardSize {
    pub title: f32,
    pub body: f32,
}

impl CardSize {
    pub fn height(self) -> i32 {
        let body = if self.body > 0.0 { 4.0 + self.body } else { 0.0 };
        (CARD_PAD * 2) + (self.title + body).ceil() as i32
    }
}

/// Visual order: top placements list the newest card first, bottom placements put it last.
pub fn display_order(count: usize, from_bottom: bool) -> Vec<usize> {
    let indices: Vec<usize> = (0..count).collect();
    if from_bottom {
        indices.into_iter().rev().collect()
    } else {
        indices
    }
}

pub fn overlay_position(
    placement: OverlayPlacement,
    work: Rect,
    overlay_width: i32,
    overlay_height: i32,
    margin: i32,
) -> (i32, i32) {
    let left = work.left + margin;
    let center_x = work.left + (work.width - overlay_width) / 2;
    let right = work.left + work.width - overlay_width - margin;
    let top = work.top + margin;
    let center_y = work.top + (work.height - overlay_height) / 2;
    let bottom = work.top + work.height - overlay_height - margin;

    match placement {
        OverlayPlacement::TopLeft => (left, top),
        OverlayPlacement::TopCenter => (center_x, top),
        OverlayPlacement::TopRight => (right, top),
        OverlayPlacement::MiddleLeft => (left, center_y),
        OverlayPlacement::Center => (center_x, center_y),
        OverlayPlacement::MiddleRight => (right, center_y),
        OverlayPlacement::BottomLeft => (left, bottom),
        OverlayPlacement::BottomCenter => (center_x, bottom),
        OverlayPlacement::BottomRight => (right, bottom),
    }
}

/// Stacks the cards and the optional "Dismiss all" row. Returns the card rectangles in
/// display order and the button rectangle when there is at least one toast.
pub fn stack_layout(
    sizes: &[CardSize],
    from_bottom: bool,
    button_width: i32,
) -> (Vec<(usize, Rect)>, Option<Rect>, i32) {
    let card_width = OVERLAY_WIDTH - PAD * 2;
    let order = display_order(sizes.len(), from_bottom);
    let mut y = PAD;
    let mut cards = Vec::with_capacity(sizes.len());
    let mut button = None;

    if !sizes.is_empty() && from_bottom {
        button = Some(Rect { left: OVERLAY_WIDTH - PAD - button_width, top: y, width: button_width, height: BUTTON_HEIGHT });
        y += BUTTON_HEIGHT + GAP;
    }
    for index in order {
        let height = sizes[index].height();
        cards.push((index, Rect { left: PAD, top: y, width: card_width, height }));
        y += height + GAP;
    }
    if !sizes.is_empty() && !from_bottom {
        button = Some(Rect { left: OVERLAY_WIDTH - PAD - button_width, top: y, width: button_width, height: BUTTON_HEIGHT });
        y += BUTTON_HEIGHT;
    } else {
        y -= GAP;
    }
    let height = y + PAD;
    (cards, button, height.max(PAD * 2))
}

pub fn close_rect(card: Rect) -> Rect {
    Rect {
        left: card.left + card.width - CLOSE_SIZE - 6,
        top: card.top + 6,
        width: CLOSE_SIZE,
        height: CLOSE_SIZE,
    }
}

/// Emoji shown before the title, so cards from different sources are easier to tell apart.
pub fn title_text(toast: &Toast) -> String {
    let badge = match toast.id.as_str() {
        DEBUG_TOAST_ID => "\u{1F6E0}",
        CAPTURE_DENIED_ID => "\u{26A0}\u{FE0F}",
        "launch" => "\u{2705}",
        _ if toast.captured => "\u{1F514}",
        _ => "",
    };
    if badge.is_empty() {
        toast.title.clone()
    } else {
        format!("{badge} {}", toast.title)
    }
}

pub fn line_height(font_size: f32) -> f32 {
    font_size * LINE_FACTOR
}

#[cfg(windows)]
pub use windows_impl::Overlay;

#[cfg(windows)]
mod windows_impl {
    use super::*;
    use std::mem::size_of;

    use windows_numerics::Vector2;
    use windows::Win32::Foundation::{COLORREF, POINT, SIZE};
    use windows::Win32::Graphics::Direct2D::Common::{
        D2D1_ALPHA_MODE_PREMULTIPLIED, D2D1_COLOR_F, D2D1_PIXEL_FORMAT, D2D_RECT_F,
    };
    use windows::Win32::Graphics::Direct2D::{
        D2D1CreateFactory, ID2D1DCRenderTarget, ID2D1Factory, ID2D1SolidColorBrush,
        D2D1_DRAW_TEXT_OPTIONS_ENABLE_COLOR_FONT, D2D1_DRAW_TEXT_OPTIONS_NONE, D2D1_FACTORY_TYPE_SINGLE_THREADED, D2D1_FEATURE_LEVEL_DEFAULT,
        D2D1_RENDER_TARGET_PROPERTIES, D2D1_RENDER_TARGET_TYPE_DEFAULT,
        D2D1_RENDER_TARGET_USAGE_NONE, D2D1_ROUNDED_RECT,
    };
    use windows::Win32::Graphics::DirectWrite::{
        DWriteCreateFactory, IDWriteFactory, IDWriteTextFormat, IDWriteTextLayout,
        DWRITE_FACTORY_TYPE_SHARED, DWRITE_FONT_STRETCH_NORMAL, DWRITE_FONT_STYLE_NORMAL,
        DWRITE_FONT_WEIGHT_BOLD, DWRITE_FONT_WEIGHT_NORMAL, DWRITE_TEXT_METRICS,
    };
    use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM;
    use windows::Win32::Graphics::Gdi::{
        CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GetDC, ReleaseDC,
        SelectObject, AC_SRC_ALPHA, AC_SRC_OVER, BITMAPINFO, BITMAPINFOHEADER, BI_RGB,
        BLENDFUNCTION, DIB_RGB_COLORS, HBITMAP, RGBQUAD,
    };
    use windows::Win32::UI::HiDpi::GetDpiForWindow;
    use windows::Win32::UI::WindowsAndMessaging::{
        GetSystemMetrics, ShowWindow, SystemParametersInfoW, UpdateLayeredWindow, SM_CXSCREEN,
        SM_CYSCREEN, SPI_GETWORKAREA, SW_HIDE, SW_SHOWNOACTIVATE, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS,
        ULW_ALPHA,
    };

    pub struct Overlay {
        hwnd: HWND,
        target: ID2D1DCRenderTarget,
        dwrite: IDWriteFactory,
        title_format: IDWriteTextFormat,
        body_format: IDWriteTextFormat,
        card_brush: ID2D1SolidColorBrush,
        button_brush: ID2D1SolidColorBrush,
        text_brush: ID2D1SolidColorBrush,
        debug_brush: ID2D1SolidColorBrush,
        card_hover_brush: ID2D1SolidColorBrush,
        glow_brush: ID2D1SolidColorBrush,
        button_hover_brush: ID2D1SolidColorBrush,
        close_hover_brush: ID2D1SolidColorBrush,
        close_format: IDWriteTextFormat,
        hits: Vec<(Hit, Rect)>,
        /// The element under the pointer, drawn with a highlight.
        hover: Option<Hit>,
        /// Device pixels per DIP for the window's monitor (1.0 at 96 DPI).
        scale: f32,
        visible: bool,
    }

    impl Overlay {
        pub fn new(hwnd: HWND) -> windows::core::Result<Self> {
            let factory: ID2D1Factory = unsafe { D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)? };
            let props = D2D1_RENDER_TARGET_PROPERTIES {
                r#type: D2D1_RENDER_TARGET_TYPE_DEFAULT,
                pixelFormat: D2D1_PIXEL_FORMAT {
                    format: DXGI_FORMAT_B8G8R8A8_UNORM,
                    alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
                },
                dpiX: 96.0,
                dpiY: 96.0,
                usage: D2D1_RENDER_TARGET_USAGE_NONE,
                minLevel: D2D1_FEATURE_LEVEL_DEFAULT,
            };
            let target = unsafe { factory.CreateDCRenderTarget(&props)? };
            let dwrite: IDWriteFactory = unsafe { DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)? };
            let close_format = unsafe {
                dwrite.CreateTextFormat(
                    w!("Segoe UI"),
                    None,
                    DWRITE_FONT_WEIGHT_NORMAL,
                    DWRITE_FONT_STYLE_NORMAL,
                    DWRITE_FONT_STRETCH_NORMAL,
                    CLOSE_FONT_SIZE,
                    w!(""),
                )?
            };
            let title_format = unsafe {
                dwrite.CreateTextFormat(
                    w!("Segoe UI"),
                    None,
                    DWRITE_FONT_WEIGHT_BOLD,
                    DWRITE_FONT_STYLE_NORMAL,
                    DWRITE_FONT_STRETCH_NORMAL,
                    TITLE_FONT_SIZE,
                    w!(""),
                )?
            };
            let body_format = unsafe {
                dwrite.CreateTextFormat(
                    w!("Segoe UI"),
                    None,
                    DWRITE_FONT_WEIGHT_NORMAL,
                    DWRITE_FONT_STYLE_NORMAL,
                    DWRITE_FONT_STRETCH_NORMAL,
                    BODY_FONT_SIZE,
                    w!(""),
                )?
            };
            let brush = |r: f32, g: f32, b: f32, a: f32| -> windows::core::Result<ID2D1SolidColorBrush> {
                let color = D2D1_COLOR_F { r, g, b, a };
                unsafe { target.CreateSolidColorBrush(&color, None) }
            };
            Ok(Self {
                hwnd,
                card_brush: brush(0.12, 0.16, 0.22, 0.94)?,
                button_brush: brush(0.22, 0.27, 0.35, 0.96)?,
                text_brush: brush(0.97, 0.98, 0.99, 1.0)?,
                debug_brush: brush(0.95, 0.25, 0.25, 0.18)?,
                card_hover_brush: brush(0.17, 0.22, 0.31, 0.97)?,
                glow_brush: brush(0.42, 0.72, 1.0, 0.45)?,
                button_hover_brush: brush(0.30, 0.36, 0.46, 0.98)?,
                close_hover_brush: brush(1.0, 1.0, 1.0, 0.20)?,
                close_format,
                target,
                dwrite,
                title_format,
                body_format,
                hits: Vec::new(),
                hover: None,
                visible: false,
                scale: 1.0,
            })
        }

        fn layout(&self, text: &str, format: &IDWriteTextFormat, width: f32) -> windows::core::Result<IDWriteTextLayout> {
            let wide: Vec<u16> = text.encode_utf16().collect();
            unsafe { self.dwrite.CreateTextLayout(&wide, format, width, 10_000.0) }
        }

        fn measure(&self, toast: &Toast) -> windows::core::Result<CardSize> {
            let text_width = (OVERLAY_WIDTH - PAD * 2 - CARD_PAD * 2 - CLOSE_SIZE) as f32;
            let title = self.layout(&title_text(toast), &self.title_format, text_width)?;
            let title_h = metrics(&title)?.height.min(line_height(TITLE_FONT_SIZE) * TITLE_LINES);
            let body_h = if toast.body.is_empty() {
                0.0
            } else {
                let body = self.layout(&toast.body, &self.body_format, text_width)?;
                metrics(&body)?.height.min(line_height(BODY_FONT_SIZE) * BODY_LINES)
            };
            Ok(CardSize { title: title_h, body: body_h })
        }

        fn button_width(&self) -> windows::core::Result<i32> {
            let label = self.layout("Dismiss all", &self.body_format, 400.0)?;
            let width = metrics(&label)?.width;
            Ok(width.ceil() as i32 + 20)
        }

        /// Draws the current toasts, or hides the window when there is nothing to show.
        pub fn render(&mut self, toasts: &[Toast], debug: bool, placement: OverlayPlacement) -> windows::core::Result<()> {
            if toasts.is_empty() && !debug {
                self.hide();
                return Ok(());
            }

            let sizes = toasts.iter().map(|toast| self.measure(toast)).collect::<windows::core::Result<Vec<_>>>()?;
            let button_width = self.button_width()?;
            let from_bottom = placement.from_bottom();
            let (cards, button, height) = stack_layout(&sizes, from_bottom, button_width);
            self.scale = self.current_scale();
            let width = OVERLAY_WIDTH;

            self.hits.clear();
            for (index, rect) in &cards {
                self.hits.push((Hit::Close(*index), close_rect(*rect)));
                self.hits.push((Hit::Card(*index), *rect));
            }
            if let Some(rect) = button {
                self.hits.push((Hit::DismissAll, rect));
            }
            let scale = self.scale;
            let (pixel_width, pixel_height) = (to_pixels(width, scale), to_pixels(height, scale));
            let (x, y) = overlay_position(placement, self.work_area(), pixel_width, pixel_height, to_pixels(MARGIN, scale));

            let screen_dc = unsafe { GetDC(None) };
            let mem_dc = unsafe { CreateCompatibleDC(Some(screen_dc)) };
            let info = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: pixel_width,
                    biHeight: -pixel_height,
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0,
                    ..Default::default()
                },
                bmiColors: [RGBQUAD::default(); 1],
            };
            let mut bits: *mut core::ffi::c_void = std::ptr::null_mut();
            let bitmap: HBITMAP = unsafe { CreateDIBSection(Some(mem_dc), &info, DIB_RGB_COLORS, &mut bits, None, 0)? };
            let old = unsafe { SelectObject(mem_dc, bitmap.into()) };

            unsafe { self.target.SetDpi(96.0 * scale, 96.0 * scale) };
            let drawn = self.draw(mem_dc, toasts, debug, pixel_width, pixel_height, &cards, button);

            let dst = POINT { x, y };
            let size = SIZE { cx: pixel_width, cy: pixel_height };
            let src = POINT { x: 0, y: 0 };
            let blend = BLENDFUNCTION {
                BlendOp: AC_SRC_OVER as u8,
                BlendFlags: 0,
                SourceConstantAlpha: 255,
                AlphaFormat: AC_SRC_ALPHA as u8,
            };
            let updated = drawn.and_then(|()| {
                unsafe {
                    let _ = ShowWindow(self.hwnd, SW_SHOWNOACTIVATE);
                    UpdateLayeredWindow(
                        self.hwnd,
                        Some(screen_dc),
                        Some(&dst),
                        Some(&size),
                        Some(mem_dc),
                        Some(&src),
                        COLORREF(0),
                        Some(&blend),
                        ULW_ALPHA,
                    )
                }
            });

            unsafe {
                SelectObject(mem_dc, old);
                let _ = DeleteObject(bitmap.into());
                let _ = DeleteDC(mem_dc);
                ReleaseDC(None, screen_dc);
            }
            self.visible = true;
            updated
        }

        fn draw(
            &self,
            mem_dc: windows::Win32::Graphics::Gdi::HDC,
            toasts: &[Toast],
            debug: bool,
            width: i32,
            height: i32,
            cards: &[(usize, Rect)],
            button: Option<Rect>,
        ) -> windows::core::Result<()> {
            unsafe {
                self.target.BindDC(
                    mem_dc,
                    &windows::Win32::Foundation::RECT { left: 0, top: 0, right: width, bottom: height },
                )?;
                self.target.BeginDraw();
                self.target.Clear(Some(&D2D1_COLOR_F { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }));
                if debug {
                    let bounds = D2D_RECT_F { left: 0.0, top: 0.0, right: width as f32 / self.scale, bottom: height as f32 / self.scale };
                    self.target.FillRectangle(&bounds, &self.debug_brush);
                }
            }

            for (index, rect) in cards {
                let toast = &toasts[*index];
                let rounded = D2D1_ROUNDED_RECT {
                    rect: to_d2d(*rect),
                    radiusX: 10.0,
                    radiusY: 10.0,
                };
                let card_hovered = matches!(self.hover, Some(Hit::Card(i)) | Some(Hit::Close(i)) if i == *index);
                if card_hovered {
                    let glow = D2D1_ROUNDED_RECT {
                        rect: to_d2d(Rect { left: rect.left - 3, top: rect.top - 3, width: rect.width + 6, height: rect.height + 6 }),
                        radiusX: 13.0,
                        radiusY: 13.0,
                    };
                    unsafe { self.target.FillRoundedRectangle(&glow, &self.glow_brush) };
                }
                let card_brush = if card_hovered { &self.card_hover_brush } else { &self.card_brush };
                unsafe { self.target.FillRoundedRectangle(&rounded, card_brush) };
                let text_left = rect.left as f32 + CARD_PAD as f32;
                let text_width = (rect.width - CARD_PAD * 2 - CLOSE_SIZE) as f32;
                let title = self.layout(&title_text(toast), &self.title_format, text_width)?;
                let title_top = rect.top as f32 + CARD_PAD as f32;
                unsafe {
                    self.target.DrawTextLayout(
                        Vector2 { X: text_left, Y: title_top },
                        &title,
                        &self.text_brush,
                        D2D1_DRAW_TEXT_OPTIONS_ENABLE_COLOR_FONT,
                    );
                }
                if !toast.body.is_empty() {
                    let body = self.layout(&toast.body, &self.body_format, text_width)?;
                    let body_top = title_top + self.measure_title_height(&title) + 4.0;
                    unsafe {
                        self.target.DrawTextLayout(
                            Vector2 { X: text_left, Y: body_top },
                            &body,
                            &self.text_brush,
                            D2D1_DRAW_TEXT_OPTIONS_ENABLE_COLOR_FONT,
                        );
                    }
                }
                let close = close_rect(*rect);
                if self.hover == Some(Hit::Close(*index)) {
                    let circle = D2D1_ROUNDED_RECT {
                        rect: to_d2d(close),
                        radiusX: CLOSE_SIZE as f32 / 2.0,
                        radiusY: CLOSE_SIZE as f32 / 2.0,
                    };
                    unsafe { self.target.FillRoundedRectangle(&circle, &self.close_hover_brush) };
                }
                let close_text = self.layout("\u{00D7}", &self.close_format, CLOSE_SIZE as f32)?;
                let glyph = metrics(&close_text)?;
                let glyph_left = close.left as f32 + (CLOSE_SIZE as f32 - glyph.width) / 2.0;
                let glyph_top = close.top as f32 + (CLOSE_SIZE as f32 - glyph.height) / 2.0;
                unsafe {
                    self.target.DrawTextLayout(
                        Vector2 { X: glyph_left, Y: glyph_top },
                        &close_text,
                        &self.text_brush,
                        D2D1_DRAW_TEXT_OPTIONS_NONE,
                    );
                }
            }

            if let Some(rect) = button {
                let rounded = D2D1_ROUNDED_RECT { rect: to_d2d(rect), radiusX: 8.0, radiusY: 8.0 };
                let brush = if self.hover == Some(Hit::DismissAll) { &self.button_hover_brush } else { &self.button_brush };
                unsafe { self.target.FillRoundedRectangle(&rounded, brush) };
                let label = self.layout("Dismiss all", &self.body_format, rect.width as f32)?;
                unsafe {
                    self.target.DrawTextLayout(
                        Vector2 { X: rect.left as f32 + 10.0, Y: rect.top as f32 + 5.0 },
                        &label,
                        &self.text_brush,
                        D2D1_DRAW_TEXT_OPTIONS_NONE,
                    );
                }
            }

            unsafe { self.target.EndDraw(None, None)? };
            Ok(())
        }

        fn current_scale(&self) -> f32 {
            match unsafe { GetDpiForWindow(self.hwnd) } {
                0 => 1.0,
                dpi => dpi as f32 / 96.0,
            }
        }

        fn measure_title_height(&self, title: &IDWriteTextLayout) -> f32 {
            metrics(title).map(|metrics| metrics.height).unwrap_or(0.0)
                .min(line_height(TITLE_FONT_SIZE) * TITLE_LINES)
        }

        pub fn hide(&mut self) {
            if self.visible {
                unsafe {
                    let _ = ShowWindow(self.hwnd, SW_HIDE);
                }
                self.visible = false;
            }
            self.hits.clear();
            self.hover = None;
        }

        /// Stores the element under the pointer. Returns true when the highlight changed and the
        /// overlay needs a redraw.
        pub fn set_hover(&mut self, hover: Option<Hit>) -> bool {
            if self.hover == hover {
                return false;
            }
            self.hover = hover;
            true
        }

        pub fn hit_test(&self, x: i32, y: i32) -> Option<Hit> {
            let scale = self.scale.max(0.1);
            let (x, y) = ((x as f32 / scale) as i32, (y as f32 / scale) as i32);
            self.hits.iter().find(|(_, rect)| rect.contains(x, y)).map(|(hit, _)| *hit)
        }

        /// Primary monitor work area (excludes the taskbar), as v2 used.
        fn work_area(&self) -> Rect {
            let mut area = RECT::default();
            let read = unsafe {
                SystemParametersInfoW(
                    SPI_GETWORKAREA,
                    0,
                    Some(&mut area as *mut RECT as *mut core::ffi::c_void),
                    SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
                )
            };
            if read.is_ok() {
                Rect {
                    left: area.left,
                    top: area.top,
                    width: area.right - area.left,
                    height: area.bottom - area.top,
                }
            } else {
                Rect {
                    left: 0,
                    top: 0,
                    width: unsafe { GetSystemMetrics(SM_CXSCREEN) },
                    height: unsafe { GetSystemMetrics(SM_CYSCREEN) },
                }
            }
        }
    }

fn metrics(layout: &IDWriteTextLayout) -> windows::core::Result<DWRITE_TEXT_METRICS> {
        let mut value = DWRITE_TEXT_METRICS::default();
        unsafe { layout.GetMetrics(&mut value)? };
        Ok(value)
    }

    fn to_pixels(value: i32, scale: f32) -> i32 {
        (value as f32 * scale).round() as i32
    }

    fn to_d2d(rect: Rect) -> D2D_RECT_F {
        D2D_RECT_F {
            left: rect.left as f32,
            top: rect.top as f32,
            right: (rect.left + rect.width) as f32,
            bottom: (rect.top + rect.height) as f32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn screen() -> Rect {
        Rect { left: 0, top: 0, width: 1920, height: 1080 }
    }

    #[test]
    fn top_right_sits_in_corner_with_margin() {
        let (x, y) = overlay_position(OverlayPlacement::TopRight, screen(), 380, 100, MARGIN);
        assert_eq!((x, y), (1920 - 380 - MARGIN, MARGIN));
    }

    #[test]
    fn bottom_center_is_horizontally_centered() {
        let (x, y) = overlay_position(OverlayPlacement::BottomCenter, screen(), 380, 100, MARGIN);
        assert_eq!(x, (1920 - 380) / 2);
        assert_eq!(y, 1080 - 100 - MARGIN);
    }

    #[test]
    fn bottom_placement_reverses_display_order() {
        assert_eq!(display_order(3, true), vec![2, 1, 0]);
        assert_eq!(display_order(3, false), vec![0, 1, 2]);
    }

    #[test]
    fn card_height_follows_title_and_body() {
        let with_body = CardSize { title: 20.0, body: 34.0 };
        let title_only = CardSize { title: 20.0, body: 0.0 };
        assert!(with_body.height() > title_only.height());
        assert_eq!(title_only.height(), CARD_PAD * 2 + 20);
    }

    #[test]
    fn stack_places_button_below_cards_at_top() {
        let sizes = [CardSize { title: 20.0, body: 0.0 }, CardSize { title: 20.0, body: 0.0 }];
        let (cards, button, height) = stack_layout(&sizes, false, 100);
        assert_eq!(cards.len(), 2);
        let button = button.expect("button");
        assert!(button.top > cards[1].1.top);
        assert_eq!(height, button.top + BUTTON_HEIGHT + PAD);
    }

    #[test]
    fn stack_places_button_above_cards_at_bottom() {
        let sizes = [CardSize { title: 20.0, body: 0.0 }];
        let (cards, button, _) = stack_layout(&sizes, true, 100);
        let button = button.expect("button");
        assert!(button.top < cards[0].1.top);
    }

    #[test]
    fn empty_stack_has_no_button() {
        let (cards, button, height) = stack_layout(&[], false, 100);
        assert!(cards.is_empty());
        assert!(button.is_none());
        assert_eq!(height, PAD * 2);
    }

    #[test]
    fn hit_test_regions_contain_points() {
        let card = Rect { left: 12, top: 12, width: 356, height: 60 };
        let close = close_rect(card);
        assert!(card.contains(20, 20));
        assert!(close.contains(close.left + 1, close.top + 1));
        assert!(!card.contains(0, 0));
    }
}
