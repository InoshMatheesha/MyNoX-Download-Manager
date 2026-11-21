//! MyNoX Island's materials: technical glass. An instrument panel under frosted glass, drawn
//! with the theme's palette (no colour of its own): a backdrop with two slow glows, translucent
//! panels with a sheen along their top, capsule buttons, chips, a segmented switch, a progress
//! ring and the dark capsule of the Dynamic Island. Numbers are set in a monospaced face
//! (JetBrains Mono), headings and labels in Space Grotesk (both bundled, `winui::install_fonts`).
//!
//! Every rect and x here is English (left to right); they are mirrored where drawn in a
//! right-to-left interface (`winui::rect`, `winui::mx`), as the rest of the windows are.

use egui::{Color32, FontFamily, FontId, Painter, Pos2, Rect, Response, Stroke, StrokeKind, Ui};
use egui::{pos2, vec2};

use crate::derive::mix;
use crate::winui;

/// The font families `winui::install_fonts` adds.
pub const DISPLAY: &str = "island-display";
pub const DISPLAY_BOLD: &str = "island-display-bold";
pub const MONO: &str = "island-mono";

/// Space Grotesk Medium.
pub fn display(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(DISPLAY.into()))
}

/// Space Grotesk Bold.
pub fn bold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(DISPLAY_BOLD.into()))
}

/// JetBrains Mono Medium: speeds, sizes, times, counts and the small capitals over them.
pub fn mono(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(MONO.into()))
}

/// Is MyNoX Island's look on?
pub fn on() -> bool {
    crate::theme::current().layout == crate::palette::Layout::Island
}

/// The baseline that centres text of `size` in a band.
pub fn mid(top: f32, h: f32, size: f32) -> f32 {
    (top + (h + size * 0.70) / 2.0).round() - 1.0
}

/// An English point where it is on screen.
pub fn at(x: f32, y: f32) -> Pos2 {
    pos2(winui::mx(x), y)
}

/// Text whose start is at English `x`; returns its width.
pub fn text(p: &Painter, f: FontId, x: f32, baseline: f32, s: &str, c: Color32) -> f32 {
    winui::text_font(p, f, x, baseline, s, c).width()
}

/// Text whose end is at English `right`; returns its width.
pub fn text_end(p: &Painter, f: FontId, right: f32, baseline: f32, s: &str, c: Color32) -> f32 {
    let w = width(p, f.clone(), s);
    winui::text_font(p, f, right - w, baseline, s, c);
    w
}

/// Text centred on English `cx`.
pub fn text_mid(p: &Painter, f: FontId, cx: f32, baseline: f32, s: &str, c: Color32) {
    let w = width(p, f.clone(), s);
    winui::text_font(p, f, cx - w / 2.0, baseline, s, c);
}

pub fn width(p: &Painter, f: FontId, s: &str) -> f32 {
    winui::text_width_font(p, f, s)
}

/// `make(size)` at the largest of `size` down to `least` (in steps of 1) at which `s` fits in
/// `max`: a long translation of a speed set smaller rather than cut.
pub fn fit(
    p: &Painter,
    make: fn(f32) -> FontId,
    size: f32,
    least: f32,
    s: &str,
    max: f32,
) -> FontId {
    let mut at = size;
    while at > least && width(p, make(at), s) > max {
        at -= 1.0;
    }
    make(at)
}

/// The inks of the glass, from the palette: the theme's light or dark variant as it is now, and
/// the Dynamic Island's own, which is always dark (the dark variant's).
pub struct Ink {
    pub dark: bool,
    pub text: Color32,
    pub muted: Color32,
    pub disabled: Color32,
    pub accent: Color32,
    /// A panel's fill, under the pointer, and its edge.
    pub glass: Color32,
    pub glass_hot: Color32,
    pub edge: Color32,
    /// The light along a panel's top.
    pub sheen: Color32,
    pub shadow: Color32,
    /// A shallow inset in the glass (the dock's pods).
    pub well: Color32,
    /// A raised thumb's face (the segmented switch).
    pub raised: Color32,
    /// The Dynamic Island: its face, ink, quiet ink and signal.
    pub island: Color32,
    pub on_island: Color32,
    pub island_muted: Color32,
    pub island_signal: Color32,
    pub island_good: Color32,
}

impl Ink {
    pub fn now() -> Self {
        let c = crate::theme::colors();
        let d = &crate::theme::current().dark;
        let dark = c.dark;
        Ink {
            dark,
            text: c.text,
            muted: c.muted,
            disabled: c.text_disabled,
            accent: c.accent,
            glass: if dark {
                Color32::from_white_alpha(11)
            } else {
                Color32::from_white_alpha(150)
            },
            glass_hot: if dark {
                Color32::from_white_alpha(22)
            } else {
                Color32::from_white_alpha(225)
            },
            edge: if dark {
                Color32::from_white_alpha(20)
            } else {
                Color32::from_black_alpha(20)
            },
            sheen: if dark {
                Color32::from_white_alpha(26)
            } else {
                Color32::from_white_alpha(235)
            },
            shadow: Color32::from_black_alpha(if dark { 120 } else { 34 }),
            well: if dark {
                Color32::from_black_alpha(46)
            } else {
                Color32::from_black_alpha(12)
            },
            raised: if dark {
                mix(c.list, c.text, 0.12)
            } else {
                Color32::WHITE
            },
            island: mix(d.window, Color32::BLACK, 0.55),
            on_island: d.text,
            island_muted: d.muted,
            island_signal: d.accent,
            island_good: d.success,
        }
    }
}

/// A soft round glow: `color` at `center`, gone at `radius`.
fn glow(p: &Painter, center: Pos2, radius: f32, color: Color32) {
    let mut mesh = egui::Mesh::default();
    let rings = [(0.0, 1.0), (0.35, 0.72), (0.65, 0.34), (1.0, 0.0)];
    let steps = 48usize;
    mesh.colored_vertex(center, color);
    for &(r, a) in rings.iter().skip(1) {
        for s in 0..steps {
            let ang = s as f32 / steps as f32 * std::f32::consts::TAU;
            let pt = center + vec2(ang.cos(), ang.sin()) * radius * r;
            mesh.colored_vertex(pt, color.gamma_multiply(a));
        }
    }
    let n = steps as u32;
    for s in 0..n {
        mesh.add_triangle(0, 1 + s, 1 + (s + 1) % n);
    }
    for ring in 0..(rings.len() as u32 - 2) {
        let (a0, b0) = (1 + ring * n, 1 + (ring + 1) * n);
        for s in 0..n {
            let s1 = (s + 1) % n;
            mesh.add_triangle(a0 + s, b0 + s, b0 + s1);
            mesh.add_triangle(a0 + s, b0 + s1, a0 + s1);
        }
    }
    p.add(egui::Shape::mesh(mesh));
}

/// The backdrop: the window's colour and two glows behind the glass, the signal colour's at the
/// leading top and the ink's at the trailing bottom. They drift slowly with `t` (seconds), so
/// they move only while the window is drawn again anyway (downloads running); nothing asks for
/// frames of its own.
pub fn backdrop(p: &Painter, rc: Rect, t: f64) {
    let c = crate::theme::colors();
    p.rect_filled(winui::rect(rc), 0.0, c.window);
    let big = rc.width().max(rc.height());
    let t = t as f32;
    let drift = |speed: f32, amp: f32, phase: f32| (t * speed + phase).sin() * amp;
    let a = rc.left() + rc.width() * 0.16 + drift(0.05, 36.0, 0.0);
    let b = rc.top() + rc.height() * 0.12 + drift(0.04, 18.0, 1.3);
    glow(
        p,
        at(a, b),
        big * 0.42,
        c.accent.gamma_multiply(if c.dark { 0.16 } else { 0.12 }),
    );
    let a = rc.right() - rc.width() * 0.10 + drift(0.03, 40.0, 2.1);
    let b = rc.bottom() - rc.height() * 0.08 + drift(0.05, 22.0, 0.4);
    glow(
        p,
        at(a, b),
        big * 0.48,
        c.text.gamma_multiply(if c.dark { 0.07 } else { 0.05 }),
    );
}

/// A frosted panel: a soft shadow, the glass, a sheen along its top and a hairline edge.
pub fn panel(p: &Painter, rc: Rect, radius: f32, k: &Ink) {
    shadow(p, rc, radius, k);
    p.rect_filled(winui::rect(rc), radius, k.glass);
    sheen(p, rc, radius, k);
    p.rect_stroke(
        winui::rect(rc),
        radius,
        Stroke::new(1.0, k.edge),
        StrokeKind::Inside,
    );
}

/// The list's card: the list's own (opaque) colour, so its rows read as they do everywhere,
/// with the glass's shadow, sheen and edge round it.
pub fn card(p: &Painter, rc: Rect, radius: f32, k: &Ink) {
    let c = crate::theme::colors();
    shadow(p, rc, radius, k);
    p.rect_filled(winui::rect(rc), radius, c.list);
    sheen(p, rc, radius, k);
    p.rect_stroke(
        winui::rect(rc),
        radius,
        Stroke::new(1.0, if k.dark { k.edge } else { c.list_frame }),
        StrokeKind::Inside,
    );
}

fn shadow(p: &Painter, rc: Rect, radius: f32, k: &Ink) {
    soft_shadow(p, rc, radius, 10.0, 30.0, k.shadow.gamma_multiply(0.55));
}

/// A shadow as light makes one: `rc` (English) blurred over `blur` points, `dy` below it, in
/// `color`, its edge fading on both sides instead of the hard step of a solid shape.
pub fn soft_shadow(p: &Painter, rc: Rect, radius: f32, dy: f32, blur: f32, color: Color32) {
    let shape =
        egui::epaint::RectShape::filled(winui::rect(rc.translate(vec2(0.0, dy))), radius, color)
            .with_blur_width(blur);
    p.add(shape);
}

/// A circle's soft shadow or glow round `center` (English).
pub fn soft_disc(p: &Painter, center: Pos2, r: f32, dy: f32, blur: f32, color: Color32) {
    let rc = Rect::from_center_size(center, vec2(r * 2.0, r * 2.0));
    soft_shadow(p, rc, r, dy, blur, color);
}

fn sheen(p: &Painter, rc: Rect, radius: f32, k: &Ink) {
    let top = Rect::from_min_size(rc.min, vec2(rc.width(), (rc.height() * 0.5).min(56.0)));
    winui::fill_shaded_corners(
        p,
        winui::rect(top),
        radius,
        0.0,
        &[
            (0.0, k.sheen.gamma_multiply(0.55)),
            (1.0, Color32::TRANSPARENT),
        ],
    );
}

/// A capsule button's kind.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// The signal colour (the window's main action).
    Primary,
    /// Glass.
    Glass,
}

/// A capsule button with its (translated) label centred; `id` keeps it apart from the others.
pub fn button(
    ui: &Ui,
    rc: Rect,
    label: &str,
    id: impl std::hash::Hash + std::fmt::Debug,
    kind: Kind,
    enabled: bool,
) -> Response {
    let p = ui.painter();
    let c = crate::theme::colors();
    let k = Ink::now();
    let r = ui.interact(
        winui::rect(rc),
        egui::Id::unique(("glass-button", id)),
        winui::pointer_only(),
    );
    winui::expose(&r, egui::Role::Button, label, enabled, None);
    let hot = enabled && r.hovered();
    let down = hot && r.is_pointer_button_down_on();
    let radius = rc.height() / 2.0;
    let ink = match kind {
        Kind::Primary if enabled => {
            let face = if down {
                c.button_accent_pressed
            } else if hot {
                c.button_accent_hot
            } else {
                c.button_accent
            };
            soft_shadow(
                p,
                rc.shrink2(vec2(6.0, 2.0)),
                radius,
                4.0,
                16.0,
                c.button_accent
                    .gamma_multiply(if hot { 0.55 } else { 0.38 }),
            );
            winui::fill_shaded(
                p,
                winui::rect(rc),
                radius,
                &[(0.0, mix(face, Color32::WHITE, 0.12)), (1.0, face)],
            );
            p.rect_stroke(
                winui::rect(rc),
                radius,
                Stroke::new(1.0, Color32::from_white_alpha(28)),
                StrokeKind::Inside,
            );
            c.button_accent_text
        }
        _ => {
            p.rect_filled(
                winui::rect(rc),
                radius,
                if down {
                    mix(k.glass_hot, k.text, 0.08)
                } else if hot {
                    k.glass_hot
                } else {
                    k.glass
                },
            );
            p.rect_stroke(
                winui::rect(rc),
                radius,
                Stroke::new(1.0, k.edge),
                StrokeKind::Inside,
            );
            if enabled { k.text } else { k.disabled }
        }
    };
    let f = display(13.0);
    let shown = winui::ellipsize(p, &f, label, rc.width() - 20.0);
    text_mid(
        p,
        f,
        rc.center().x,
        mid(rc.top(), rc.height(), 13.0),
        &shown,
        ink,
    );
    r
}

/// A chip: a word that picks something. Off, it is only its word in the quiet ink; on, a soft
/// signal-tinted capsule with a dot before the word. `None` for `on`: a toggle shown greyed.
pub fn chip(
    ui: &Ui,
    rc: Rect,
    id: impl std::hash::Hash + std::fmt::Debug,
    label: &str,
    on: bool,
    dim: bool,
) -> Response {
    let p = ui.painter();
    let k = Ink::now();
    let r = ui.interact(
        winui::rect(rc),
        egui::Id::unique(("glass-chip", id)),
        winui::pointer_only(),
    );
    winui::expose(&r, egui::Role::CheckBox, label, true, Some(on));
    let radius = rc.height() / 2.0;
    if on {
        p.rect_filled(winui::rect(rc), radius, k.accent.gamma_multiply(0.16));
        p.rect_stroke(
            winui::rect(rc),
            radius,
            Stroke::new(1.0, k.accent.gamma_multiply(0.45)),
            StrokeKind::Inside,
        );
        p.circle_filled(at(rc.left() + 13.0, rc.center().y), 3.0, k.accent);
    } else if r.hovered() {
        p.rect_filled(winui::rect(rc), radius, k.glass_hot);
    }
    let ink = if on {
        k.text
    } else if dim {
        k.disabled
    } else {
        k.muted
    };
    let f = display(12.5);
    let x = rc.left() + if on { 22.0 } else { 12.0 };
    text(p, f, x, mid(rc.top(), rc.height(), 12.5), label, ink);
    r
}

/// A chip's width for `label`.
pub fn chip_w(p: &Painter, label: &str) -> f32 {
    (width(p, display(12.5), label) + 34.0).ceil()
}

/// The segmented switch: a glass track and a raised thumb that slides to the chosen segment
/// (`None`: none is, the thumb hidden). Returns the segment clicked and the switch's width.
pub fn segmented(
    ui: &Ui,
    x: f32,
    y: f32,
    h: f32,
    labels: &[&str],
    current: Option<usize>,
    id: &str,
) -> (Option<usize>, f32) {
    let p = ui.painter();
    let c = crate::theme::colors();
    let k = Ink::now();
    let f = display(12.5);
    let widths: Vec<f32> = labels
        .iter()
        .map(|l| (width(p, f.clone(), l) + 30.0).ceil())
        .collect();
    let total = widths.iter().sum::<f32>() + 8.0;
    let track = Rect::from_min_size(pos2(x, y), vec2(total, h));
    p.rect_filled(winui::rect(track), h / 2.0, k.glass);
    p.rect_stroke(
        winui::rect(track),
        h / 2.0,
        Stroke::new(1.0, k.edge),
        StrokeKind::Inside,
    );
    let starts: Vec<f32> = widths
        .iter()
        .scan(track.left() + 4.0, |s, w| {
            let at = *s;
            *s += w;
            Some(at)
        })
        .collect();
    let ctx = ui.ctx();
    let key = egui::Id::unique(("glass-seg", id));
    if let Some(i) = current {
        // The thumb glides to its segment.
        let tx = ctx.animate_value_with_time(key.with("x"), starts[i], 0.22);
        let tw = ctx.animate_value_with_time(key.with("w"), widths[i], 0.22);
        let thumb = Rect::from_min_size(pos2(tx, y + 4.0), vec2(tw, h - 8.0));
        let tr = (h - 8.0) / 2.0;
        soft_shadow(p, thumb, tr, 1.5, 6.0, k.shadow.gamma_multiply(0.45));
        winui::fill_shaded(
            p,
            winui::rect(thumb),
            tr,
            &[(0.0, mix(k.raised, Color32::WHITE, 0.04)), (1.0, k.raised)],
        );
        p.rect_stroke(
            winui::rect(thumb),
            tr,
            Stroke::new(
                1.0,
                if k.dark {
                    Color32::from_white_alpha(16)
                } else {
                    c.list_frame
                },
            ),
            StrokeKind::Inside,
        );
    }
    let mut picked = None;
    for (i, label) in labels.iter().enumerate() {
        let rc = Rect::from_min_size(pos2(starts[i], y + 4.0), vec2(widths[i], h - 8.0));
        let r = ui.interact(winui::rect(rc), key.with(("seg", i)), winui::pointer_only());
        winui::expose(&r, egui::Role::Tab, label, true, Some(current == Some(i)));
        let ink = if current == Some(i) {
            k.text
        } else if r.hovered() {
            mix(k.muted, k.text, 0.5)
        } else {
            k.muted
        };
        text_mid(
            p,
            f.clone(),
            rc.center().x,
            mid(rc.top(), rc.height(), 12.5),
            label,
            ink,
        );
        if r.clicked() {
            picked = Some(i);
        }
    }
    (picked, total)
}

/// A switch: a track and a knob, on or off.
pub fn switch(p: &Painter, rc: Rect, on: bool, k: &Ink) {
    let c = crate::theme::colors();
    p.rect_filled(
        winui::rect(rc),
        rc.height() / 2.0,
        if on { k.accent } else { c.list_frame },
    );
    let r = rc.height() / 2.0 - 3.0;
    let kx = if on {
        rc.right() - rc.height() / 2.0
    } else {
        rc.left() + rc.height() / 2.0
    };
    soft_disc(
        p,
        pos2(kx, rc.center().y),
        r,
        1.0,
        4.0,
        k.shadow.gamma_multiply(0.6),
    );
    p.circle_filled(at(kx, rc.center().y), r, Color32::WHITE);
}

/// A crescent moon of radius `r` round `center` (English): the disc less a smaller one up and
/// toward the trailing side, filled as a shape so nothing behind it has to be matched.
pub fn moon(p: &Painter, center: Pos2, r: f32, ink: Color32) {
    use std::f32::consts::TAU;
    let c = at(center.x, center.y);
    let dir = if winui::rtl() { -1.0 } else { 1.0 };
    let o = vec2(r * 0.62 * dir, -r * 0.48);
    let ri = r * 0.86;
    let d = o.length();
    let a = (r * r - ri * ri + d * d) / (2.0 * d);
    let h = (r * r - a * a).max(0.0).sqrt();
    let u = o / d;
    let n = vec2(-u.y, u.x);
    let (p1, p2) = (c + u * a + n * h, c + u * a - n * h);
    let away = (-u.y).atan2(-u.x);
    // The arc of the circle round `cc` from p1 to p2 that passes the side away from the bite.
    let arc = |cc: Pos2, rad: f32| -> Vec<Pos2> {
        let (s0, e0) = (
            (p1.y - cc.y).atan2(p1.x - cc.x),
            (p2.y - cc.y).atan2(p2.x - cc.x),
        );
        let sweep = (e0 - s0).rem_euclid(TAU);
        let through = (away - s0).rem_euclid(TAU) <= sweep;
        let span = if through { sweep } else { sweep - TAU };
        (0..=32)
            .map(|k| {
                let t = s0 + span * k as f32 / 32.0;
                cc + vec2(t.cos(), t.sin()) * rad
            })
            .collect()
    };
    let (outer, inner) = (arc(c, r), arc(c + o, ri));
    let mut mesh = egui::Mesh::default();
    for (a, b) in outer.iter().zip(&inner) {
        mesh.colored_vertex(*a, ink);
        mesh.colored_vertex(*b, ink);
    }
    for k in 0..32u32 {
        let i = k * 2;
        mesh.add_triangle(i, i + 1, i + 2);
        mesh.add_triangle(i + 1, i + 3, i + 2);
    }
    p.add(egui::Shape::mesh(mesh));
}

/// A progress ring round `center` (English): its track, and `frac` of it from the top, clockwise
/// (counter-clockwise in a right-to-left interface, as it reads there). `None`: a short arc
/// that runs round while `t` goes on.
#[allow(clippy::too_many_arguments)]
pub fn ring(
    p: &Painter,
    center: Pos2,
    radius: f32,
    thick: f32,
    frac: Option<f32>,
    t: f64,
    track: Color32,
    ink: Color32,
) {
    let c = at(center.x, center.y);
    p.circle_stroke(c, radius, Stroke::new(thick, track));
    let dir = if winui::rtl() { -1.0 } else { 1.0 };
    let (from, span) = match frac {
        Some(f) => (0.0, f.clamp(0.0, 1.0)),
        None => (((t * 0.8) % 1.0) as f32, 0.22),
    };
    if span <= 0.0 {
        return;
    }
    let steps = ((span * 96.0).ceil() as usize).max(2);
    let pts: Vec<Pos2> = (0..=steps)
        .map(|s| {
            let a = (from + span * s as f32 / steps as f32) * std::f32::consts::TAU
                - std::f32::consts::FRAC_PI_2;
            c + vec2(a.cos() * dir, a.sin()) * radius
        })
        .collect();
    if let (Some(a), Some(b)) = (pts.first(), pts.last()) {
        p.circle_filled(*a, thick / 2.0, ink);
        p.circle_filled(*b, thick / 2.0, ink);
    }
    p.add(egui::Shape::line(pts, Stroke::new(thick, ink)));
}

/// A minute or so of speeds as a line with a soft fill under it, the newest at the trailing
/// edge; a flat hairline while nothing came.
pub fn sparkline(p: &Painter, rc: Rect, samples: &[f32], slots: usize, ink: Color32, k: &Ink) {
    let peak = samples.iter().copied().fold(0.0f32, f32::max);
    if samples.len() < 2 || peak <= 0.0 {
        let y = rc.bottom() - 1.0;
        p.line_segment(
            [at(rc.left(), y), at(rc.right(), y)],
            Stroke::new(1.0, k.edge),
        );
        return;
    }
    let step = rc.width() / (slots.max(2) - 1) as f32;
    let x0 = rc.right() - step * (samples.len() - 1) as f32;
    let pts: Vec<Pos2> = samples
        .iter()
        .enumerate()
        .map(|(i, v)| {
            at(
                x0 + step * i as f32,
                rc.bottom() - v / peak * (rc.height() - 2.0),
            )
        })
        .collect();
    let mut mesh = egui::Mesh::default();
    let (top, bottom) = (ink.gamma_multiply(0.30), ink.gamma_multiply(0.0));
    for w in pts.windows(2) {
        let base = mesh.vertices.len() as u32;
        mesh.colored_vertex(w[0], top);
        mesh.colored_vertex(w[1], top);
        mesh.colored_vertex(pos2(w[1].x, rc.bottom()), bottom);
        mesh.colored_vertex(pos2(w[0].x, rc.bottom()), bottom);
        mesh.add_triangle(base, base + 1, base + 2);
        mesh.add_triangle(base, base + 2, base + 3);
    }
    p.add(egui::Shape::mesh(mesh));
    p.add(egui::Shape::line(pts.clone(), Stroke::new(1.6, ink)));
    if let Some(last) = pts.last() {
        p.circle_filled(*last, 2.6, ink);
    }
}

/// A small label over a control (`over`, a screen rect) while the pointer rests on it: the
/// Dynamic Island's dark capsule, above everything else in the window.
pub fn tip(ui: &Ui, over: Rect, label: &str) {
    let k = Ink::now();
    let p = ui.ctx().layer_painter(egui::LayerId::new(
        egui::Order::Tooltip,
        egui::Id::unique("glass-tip"),
    ));
    let f = display(12.0);
    let w = width(&p, f.clone(), label) + 22.0;
    let h = 26.0;
    let rc = Rect::from_min_size(
        pos2(over.center().x - w / 2.0, over.top() - h - 8.0),
        vec2(w, h),
    );
    p.add(
        egui::epaint::RectShape::filled(rc.translate(vec2(0.0, 3.0)), h / 2.0, k.shadow)
            .with_blur_width(12.0),
    );
    p.rect_filled(rc, h / 2.0, k.island);
    let g = p.layout_no_wrap(label.to_owned(), f, k.on_island);
    let pos = pos2(
        rc.center().x - g.size().x / 2.0,
        rc.center().y - g.size().y / 2.0,
    );
    p.galley(pos, g, k.on_island);
}

/// The text in small capitals over a value (an eyebrow): the quiet ink, monospaced.
pub fn eyebrow(p: &Painter, x: f32, baseline: f32, s: &str, max: f32, k: &Ink) -> f32 {
    let f = mono(10.0);
    let s = winui::ellipsize(p, &f, &s.to_uppercase(), max);
    text(p, f, x, baseline, &s, k.muted)
}
