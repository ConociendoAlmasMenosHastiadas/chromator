use eframe::egui;

// ── Color conversions ──────────────────────────────────────────────

/// HSV → RGB  (all values 0.0–1.0)
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let c = v * s;
    let h_prime = (h * 360.0) / 60.0;
    let x = c * (1.0 - ((h_prime % 2.0) - 1.0).abs());
    let (r1, g1, b1) = match h_prime as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = v - c;
    (r1 + m, g1 + m, b1 + m)
}

/// RGB (0.0–1.0) → HSV (h 0.0–1.0, s 0.0–1.0, v 0.0–1.0)
#[allow(dead_code)]
fn rgb_to_hsv(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let h = if delta == 0.0 {
        0.0
    } else if max == r {
        (((g - b) / delta) % 6.0 + 6.0) % 6.0 / 6.0
    } else if max == g {
        ((b - r) / delta + 2.0) / 6.0
    } else {
        ((r - g) / delta + 4.0) / 6.0
    };
    let s = if max == 0.0 { 0.0 } else { delta / max };
    (h, s, max)
}

/// RGB (0.0–1.0) → HSL (h 0.0–1.0, s 0.0–1.0, l 0.0–1.0)
fn rgb_to_hsl(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    let delta = max - min;

    if delta == 0.0 {
        return (0.0, 0.0, l);
    }

    let s = if l < 0.5 {
        delta / (max + min)
    } else {
        delta / (2.0 - max - min)
    };

    let h = if max == r {
        (((g - b) / delta) % 6.0 + 6.0) % 6.0 / 6.0
    } else if max == g {
        ((b - r) / delta + 2.0) / 6.0
    } else {
        ((r - g) / delta + 4.0) / 6.0
    };

    (h, s, l)
}

/// RGB (0.0–1.0) → CMYK (0.0–1.0 each)
fn rgb_to_cmyk(r: f32, g: f32, b: f32) -> (f32, f32, f32, f32) {
    let k = 1.0 - r.max(g).max(b);
    if k >= 1.0 {
        return (0.0, 0.0, 0.0, 1.0);
    }
    let c = (1.0 - r - k) / (1.0 - k);
    let m = (1.0 - g - k) / (1.0 - k);
    let y = (1.0 - b - k) / (1.0 - k);
    (c, m, y, k)
}

// ── App state ──────────────────────────────────────────────────────

struct Chromator {
    hue: f32, // 0.0–1.0
    sat: f32, // 0.0–1.0
    val: f32, // 0.0–1.0
    sv_texture: Option<egui::TextureHandle>,
    sv_texture_hue: f32,
}

impl Default for Chromator {
    fn default() -> Self {
        Self {
            hue: 0.0,
            sat: 1.0,
            val: 1.0,
            sv_texture: None,
            sv_texture_hue: -1.0,
        }
    }
}

impl eframe::App for Chromator {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Chromator");
            ui.add_space(8.0);

            let (r, g, b) = hsv_to_rgb(self.hue, self.sat, self.val);

            // ── Top row: color swatch (left) + SV plane (right) ────
            let avail_width = ui.available_width();
            let swatch_width = 240.0;
            let sv_width = (avail_width - swatch_width - 32.0).max(200.0);
            let sv_height = 400.0;

            ui.horizontal(|ui| {
                // Color swatch
                let (swatch_rect, _) =
                    ui.allocate_exact_size(egui::vec2(swatch_width, sv_height), egui::Sense::hover());
                ui.painter().rect_filled(
                    swatch_rect,
                    4.0,
                    egui::Color32::from_rgb(
                        (r * 255.0) as u8,
                        (g * 255.0) as u8,
                        (b * 255.0) as u8,
                    ),
                );
                // Border around swatch
                ui.painter().rect_stroke(
                    swatch_rect,
                    4.0,
                    egui::Stroke::new(1.0, egui::Color32::GRAY),
                    egui::StrokeKind::Outside,
                );

                // SV plane
                self.draw_sv_plane(ui, sv_width, sv_height);
            });

            ui.add_space(8.0);

            // ── Hue bar ────────────────────────────────────────────
            self.draw_hue_bar(ui, avail_width);

            ui.add_space(12.0);

            // ── Color values (plain text) ──────────────────────────
            let r8 = (r * 255.0).round() as u8;
            let g8 = (g * 255.0).round() as u8;
            let b8 = (b * 255.0).round() as u8;

            let (c, m, y, k) = rgb_to_cmyk(r, g, b);
            let (hsl_h, hsl_s, hsl_l) = rgb_to_hsl(r, g, b);

            ui.monospace(format!("RGB:  ({}, {}, {})", r8, g8, b8));
            ui.monospace(format!(
                "CMYK: ({:.2}, {:.2}, {:.2}, {:.2})",
                c, m, y, k
            ));
            ui.monospace(format!(
                "HSV:  ({:.1}°, {:.1}%, {:.1}%)",
                self.hue * 360.0,
                self.sat * 100.0,
                self.val * 100.0
            ));
            ui.monospace(format!(
                "HSL:  ({:.1}°, {:.1}%, {:.1}%)",
                hsl_h * 360.0,
                hsl_s * 100.0,
                hsl_l * 100.0
            ));
            ui.monospace(format!("Hex:  #{:02X}{:02X}{:02X}", r8, g8, b8));
        });
    }
}

impl Chromator {
    /// Draw the Saturation-Value plane for the current hue.
    fn draw_sv_plane(&mut self, ui: &mut egui::Ui, width: f32, height: f32) {
        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::click_and_drag());

        // Handle interaction
        if response.clicked() || response.dragged() {
            if let Some(pos) = response.interact_pointer_pos() {
                self.sat = ((pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
                self.val = 1.0 - ((pos.y - rect.top()) / rect.height()).clamp(0.0, 1.0);
            }
        }

        // Regenerate texture when hue changes
        let tex_size = 256;
        if self.sv_texture.is_none() || self.sv_texture_hue != self.hue {
            let mut pixels = Vec::with_capacity(tex_size * tex_size);
            for row in 0..tex_size {
                for col in 0..tex_size {
                    let s = col as f32 / (tex_size - 1) as f32;
                    let v = 1.0 - row as f32 / (tex_size - 1) as f32;
                    let (cr, cg, cb) = hsv_to_rgb(self.hue, s, v);
                    pixels.push(egui::Color32::from_rgb(
                        (cr * 255.0) as u8,
                        (cg * 255.0) as u8,
                        (cb * 255.0) as u8,
                    ));
                }
            }
            let image = egui::ColorImage {
                size: [tex_size, tex_size],
                pixels,
            };
            let tex = ui.ctx().load_texture("sv_plane", image, egui::TextureOptions::LINEAR);
            self.sv_texture = Some(tex);
            self.sv_texture_hue = self.hue;
        }

        // Paint the texture
        if let Some(tex) = &self.sv_texture {
            let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
            ui.painter().image(tex.id(), rect, uv, egui::Color32::WHITE);
        }

        // Crosshair at current (sat, val)
        let cx = rect.left() + self.sat * rect.width();
        let cy = rect.top() + (1.0 - self.val) * rect.height();
        let crosshair_color = if self.val > 0.5 {
            egui::Color32::BLACK
        } else {
            egui::Color32::WHITE
        };
        ui.painter().circle_stroke(
            egui::pos2(cx, cy),
            10.0,
            egui::Stroke::new(3.0, crosshair_color),
        );
    }

    /// Draw the horizontal hue bar.
    fn draw_hue_bar(&mut self, ui: &mut egui::Ui, width: f32) {
        let bar_height = 48.0;
        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(width, bar_height), egui::Sense::click_and_drag());

        if response.clicked() || response.dragged() {
            if let Some(pos) = response.interact_pointer_pos() {
                self.hue = ((pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
            }
        }

        let painter = ui.painter_at(rect);
        let segments = 256_u32;
        let seg_w = rect.width() / segments as f32;

        for i in 0..segments {
            let h = i as f32 / segments as f32;
            let (cr, cg, cb) = hsv_to_rgb(h, 1.0, 1.0);
            let color = egui::Color32::from_rgb(
                (cr * 255.0) as u8,
                (cg * 255.0) as u8,
                (cb * 255.0) as u8,
            );
            let seg_rect = egui::Rect::from_min_size(
                egui::pos2(rect.left() + i as f32 * seg_w, rect.top()),
                egui::vec2(seg_w + 0.5, bar_height),
            );
            painter.rect_filled(seg_rect, 0.0, color);
        }

        // Indicator line at current hue
        let hx = rect.left() + self.hue * rect.width();
        painter.line_segment(
            [egui::pos2(hx, rect.top()), egui::pos2(hx, rect.bottom())],
            egui::Stroke::new(2.0, egui::Color32::WHITE),
        );
        painter.line_segment(
            [egui::pos2(hx, rect.top()), egui::pos2(hx, rect.bottom())],
            egui::Stroke::new(1.0, egui::Color32::BLACK),
        );
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1120.0, 820.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Chromator",
        options,
        Box::new(|_cc| Ok(Box::new(Chromator::default()))),
    )
}
