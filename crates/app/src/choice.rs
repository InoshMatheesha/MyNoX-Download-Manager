//! MyNoX Choice's main window: midnight sidebar, header, action bar, list card and status bar.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use egui::{Color32, FontFamily, FontId, Pos2, Rect, Stroke, StrokeKind, Ui, pos2, vec2};

use super::{App, Branch, Cmd, Popup, View};
use crate::chrome::{self, IconKey, TreeEvent, TreeRow};
use crate::derive::mix;
use crate::model::Status;
use crate::winui;

// An action bar button: icon, label, enabled, command, drop-down (Some(start queue?)), group.
type Action = (&'static str, &'static str, bool, Cmd, Option<bool>, u8);

// Sidebar width, open and folded to a rail.
pub(super) const SIDE_W: f32 = 232.0;
pub(super) const RAIL_W: f32 = 68.0;
// Header, action bar and status bar heights.
const HEADER_H: f32 = 70.0;
const ACTIONS_H: f32 = 36.0;
const STATUS_H: f32 = 32.0;
// One minute of speed samples for the sparkline.
const SAMPLES: usize = 60;

// What the layout keeps between frames.
#[derive(Default)]
pub(super) struct State {
    nav_scroll: f32,
    ext: Option<(Instant, bool)>,
    speeds: VecDeque<f32>,
    sampled: Option<Instant>,
}

// The sidebar's inks: always midnight, in both modes.
struct Ink {
    side: Color32,
    text: Color32,
    muted: Color32,
    faint: Color32,
    hover: Color32,
    accent: Color32,
    violet: Color32,
    good: Color32,
    warn: Color32,
}

impl Ink {
    fn now() -> Self {
        let c = crate::theme::colors();
        Ink {
            side: c.caption,
            text: Color32::from_rgb(0xe9, 0xee, 0xfb),
            muted: Color32::from_rgb(0x8d, 0x97, 0xb5),
            faint: Color32::WHITE.gamma_multiply(0.07),
            hover: Color32::WHITE.gamma_multiply(0.06),
            accent: Color32::from_rgb(0x4c, 0x82, 0xff),
            violet: Color32::from_rgb(0x8a, 0x6b, 0xff),
            good: Color32::from_rgb(0x45, 0xdc, 0xa0),
            warn: Color32::from_rgb(0xff, 0xc8, 0x57),
        }
    }
}

// Segoe UI at a size and weight (the families winui::install_fonts adds).
fn font(size: f32, weight: u8) -> FontId {
    match weight {
        0 => FontId::proportional(size),
        1 => FontId::new(size, FontFamily::Name("segoe-semibold".into())),
        _ => FontId::new(size, FontFamily::Name("segoe-bold".into())),
    }
}

// The baseline that centres text of `size` in a band.
fn mid(top: f32, h: f32, size: f32) -> f32 {
    (top + (h + size * 0.70) / 2.0).round() - 1.0
}

// An English point where it is on screen (mirrored right to left).
fn at(x: f32, y: f32) -> Pos2 {
    pos2(winui::mx(x), y)
}

// Text whose start is at English x, returns its width.
fn text(p: &egui::Painter, f: FontId, x: f32, baseline: f32, s: &str, c: Color32) -> f32 {
    winui::text_font(p, f, x, baseline, s, c).width()
}

fn width(p: &egui::Painter, f: FontId, s: &str) -> f32 {
    winui::text_width_font(p, f, s)
}

// A one-colour icon of the theme's set in `ink`, inside `rc` (English).
fn glyph(ui: &Ui, p: &egui::Painter, need: &str, rc: Rect, ink: Color32) {
    let px = rc.width() * ui.ctx().pixels_per_point();
    if let Some((src, tint)) = crate::themeicons::source(need, px, ink, true) {
        egui::Image::new(src)
            .tint(tint)
            .paint_at(ui, winui::rect(rc).intersect(p.clip_rect()));
    }
}

// The icon-set name for a tree row's icon.
fn need_of(icon: &IconKey) -> &'static str {
    match icon {
        IconKey::Themed(need, _) => need,
        _ => "cat-queues",
    }
}

// A small chevron pointing down (open) or forward (shut), centred on `c` (English).
fn chevron(p: &egui::Painter, c: Pos2, open: bool, ink: Color32) {
    let s = Stroke::new(1.6, ink);
    let pts = if open {
        [
            at(c.x - 4.0, c.y - 2.0),
            at(c.x, c.y + 2.0),
            at(c.x + 4.0, c.y - 2.0),
        ]
    } else {
        [
            at(c.x - 2.0, c.y - 4.0),
            at(c.x + 2.0, c.y),
            at(c.x - 2.0, c.y + 4.0),
        ]
    };
    p.line_segment([pts[0], pts[1]], s);
    p.line_segment([pts[1], pts[2]], s);
}

// A rounded pill with a count in it.
fn pill(p: &egui::Painter, x: f32, cy: f32, label: &str, bg: Color32, ink: Color32) -> f32 {
    let f = font(11.0, 1);
    let w = width(p, f.clone(), label).max(8.0) + 14.0;
    let rc = Rect::from_min_size(pos2(x, cy - 9.0), vec2(w, 18.0));
    p.rect_filled(winui::rect(rc), 9.0, bg);
    text(p, f, x + 7.0, mid(rc.top(), 18.0, 11.0), label, ink);
    w
}

impl App {
    // The sidebar's width: the library, or a rail of icons while Categories are hidden.
    pub(super) fn choice_side_w(&self) -> f32 {
        if self.show_categories { SIDE_W } else { RAIL_W }
    }

    // Paints the sidebar, header, action bar, list card and status bar; returns the list's rect.
    pub(super) fn choice_chrome(
        &mut self,
        ui: &mut Ui,
        o: Pos2,
        w: f32,
        h: f32,
        chosen: &mut Option<(Cmd, Option<u64>)>,
    ) -> Rect {
        let side = self.choice_side_w();
        let (cx, cw) = (o.x + side, w - side);
        let top = o.y + chrome::MENU_H;
        self.choice_sidebar(ui, o, side, h, chosen);
        self.choice_header(ui, cx, top, cw, chosen);
        self.choice_actions(ui, cx, top + HEADER_H, cw, chosen);
        let card = Rect::from_min_max(
            pos2(cx + 18.0, top + HEADER_H + ACTIONS_H + 14.0),
            pos2(o.x + w - 18.0, o.y + h - STATUS_H),
        );
        let p = ui.painter();
        let c = crate::theme::colors();
        // A soft shadow under the card, then the card.
        p.rect_filled(
            winui::rect(card.translate(vec2(0.0, 2.0))),
            14.0,
            Color32::from_black_alpha(if c.dark { 60 } else { 14 }),
        );
        p.rect_filled(winui::rect(card), 14.0, c.list);
        p.rect_stroke(
            winui::rect(card),
            14.0,
            Stroke::new(1.0, c.list_frame),
            StrokeKind::Inside,
        );
        self.choice_status(ui, cx, o.y + h - STATUS_H, cw);
        card.shrink2(vec2(8.0, 8.0))
    }

    fn choice_sidebar(
        &mut self,
        ui: &mut Ui,
        o: Pos2,
        side: f32,
        h: f32,
        chosen: &mut Option<(Cmd, Option<u64>)>,
    ) {
        let ctx = ui.ctx().clone();
        let p = ui.painter().clone();
        let k = Ink::now();
        let rail = side < SIDE_W;
        let pad = if rail { 12.0 } else { 16.0 };
        let bar = Rect::from_min_size(o, vec2(side, h));
        p.rect_filled(winui::rect(bar), 0.0, k.side);
        // A faint blue glow at the top of the sidebar.
        winui::fill_shaded(
            &p,
            winui::rect(Rect::from_min_size(o, vec2(side, 180.0))),
            0.0,
            &[
                (0.0, k.accent.gamma_multiply(0.10)),
                (1.0, Color32::TRANSPARENT),
            ],
        );

        // The brand: logo, name and what it is.
        let lx = if rail {
            o.x + (side - 32.0) / 2.0
        } else {
            o.x + 18.0
        };
        let logo = Rect::from_min_size(pos2(lx, o.y + 14.0), vec2(32.0, 32.0));
        egui::Image::new(egui::include_image!("../assets/icons/app/mynox-64.png"))
            .paint_at(ui, winui::rect(logo));
        if !rail {
            text(&p, font(17.0, 2), o.x + 58.0, o.y + 30.0, "MyNoX", k.text);
            // The rest of the product's name, never translated.
            text(
                &p,
                font(11.0, 0),
                o.x + 58.0,
                o.y + 44.0,
                "Download Manager",
                k.muted,
            );
        }

        // Add URL: the one big button.
        let add = Rect::from_min_size(pos2(o.x + pad, o.y + 62.0), vec2(side - 2.0 * pad, 42.0));
        // The toolbar's own words (already translated).
        let add_label = crate::i18n::tc("toolbar", "Add URL");
        let r = ui.interact(
            winui::rect(add),
            egui::Id::unique("choice-add"),
            winui::pointer_only(),
        );
        winui::expose(&r, egui::Role::Button, add_label, true, None);
        let r = if rail { r.on_hover_text(add_label) } else { r };
        let lift = if r.is_pointer_button_down_on() {
            -0.10
        } else if r.hovered() {
            0.10
        } else {
            0.0
        };
        let tone = |c: Color32| {
            if lift >= 0.0 {
                mix(c, Color32::WHITE, lift)
            } else {
                mix(c, Color32::BLACK, -lift)
            }
        };
        p.rect_filled(
            winui::rect(add.translate(vec2(0.0, 3.0))),
            12.0,
            k.accent.gamma_multiply(0.35),
        );
        winui::fill_shaded(
            &p,
            winui::rect(add),
            11.0,
            &[
                (0.0, tone(k.accent)),
                (1.0, tone(mix(k.accent, k.violet, 0.55))),
            ],
        );
        let plus_x = if rail {
            add.center().x
        } else {
            add.left() + 22.0
        };
        let cy = add.center().y;
        let s = Stroke::new(2.2, Color32::WHITE);
        p.line_segment([at(plus_x - 6.0, cy), at(plus_x + 6.0, cy)], s);
        p.line_segment([at(plus_x, cy - 6.0), at(plus_x, cy + 6.0)], s);
        if !rail {
            text(
                &p,
                font(14.0, 1),
                add.left() + 40.0,
                mid(add.top(), add.height(), 14.0),
                add_label,
                Color32::WHITE,
            );
            let hint = "Ctrl+N";
            let f = font(11.0, 0);
            let hw = width(&p, f.clone(), hint);
            text(
                &p,
                f,
                add.right() - 12.0 - hw,
                mid(add.top(), add.height(), 11.0),
                hint,
                Color32::WHITE.gamma_multiply(0.65),
            );
        }
        if r.clicked() {
            *chosen = Some((Cmd::AddUrl, None));
        }

        // Add Torrent with its drop-down (magnet link, RSS feeds, torrent settings).
        let tor = Rect::from_min_size(pos2(o.x + pad, o.y + 112.0), vec2(side - 2.0 * pad, 34.0));
        let tor_label = crate::i18n::tc("toolbar", "Add Torrent");
        let arrow_w = if rail { 0.0 } else { 30.0 };
        let main = Rect::from_min_max(tor.min, pos2(tor.right() - arrow_w, tor.bottom()));
        let r = ui.interact(
            winui::rect(main),
            egui::Id::unique("choice-torrent"),
            winui::pointer_only(),
        );
        winui::expose(&r, egui::Role::Button, tor_label, true, None);
        let r = if rail { r.on_hover_text(tor_label) } else { r };
        p.rect_filled(winui::rect(tor), 10.0, k.faint);
        if r.hovered() {
            p.rect_filled(winui::rect(main), 10.0, k.hover);
        }
        let ix = if rail {
            tor.center().x - 9.0
        } else {
            tor.left() + 12.0
        };
        glyph(
            ui,
            &p,
            "tb-add-torrent",
            Rect::from_min_size(pos2(ix, tor.center().y - 9.0), vec2(18.0, 18.0)),
            k.text,
        );
        if !rail {
            text(
                &p,
                font(13.0, 1),
                tor.left() + 38.0,
                mid(tor.top(), tor.height(), 13.0),
                tor_label,
                k.text,
            );
            let arrow = Rect::from_min_max(pos2(tor.right() - arrow_w, tor.top()), tor.max);
            let ar = ui.interact(
                winui::rect(arrow),
                egui::Id::unique("choice-torrent-more"),
                winui::pointer_only(),
            );
            // i18n: MyNoX Choice sidebar: tooltip of the arrow beside Add Torrent (magnet links, RSS feeds...)
            let ar = ar.on_hover_text(crate::i18n::t("More torrent options"));
            if ar.hovered() {
                p.rect_filled(winui::rect(arrow), 10.0, k.hover);
            }
            p.line_segment(
                [
                    at(arrow.left(), tor.top() + 8.0),
                    at(arrow.left(), tor.bottom() - 8.0),
                ],
                Stroke::new(1.0, k.faint),
            );
            chevron(&p, arrow.center(), true, k.muted);
            if ar.clicked() {
                self.open_popup(Popup::TorrentDrop, at(arrow.left(), arrow.bottom() + 2.0));
            }
        }
        if r.clicked() {
            *chosen = Some((Cmd::OpenTorrentFile, None));
        }

        // The bottom: extension, speed card (when the library keeps its room) and small buttons.
        let label_y = o.y + 166.0;
        let foot_h = if rail { 4.0 * 40.0 + 8.0 } else { 48.0 };
        let ext_h = 38.0;
        let foot_top = o.y + h - foot_h;
        let want = self.choice_nav_height(rail).min(300.0);
        let card_room = foot_top - 8.0 - ext_h - 8.0 - (label_y + 8.0) - (96.0 + 8.0);
        let card_h = if !rail && card_room >= want {
            96.0
        } else {
            0.0
        };
        let card_top = foot_top - card_h - if card_h > 0.0 { 8.0 } else { 0.0 };
        let ext_top = card_top - ext_h - 8.0;
        self.choice_extension(
            ui,
            &p,
            &k,
            Rect::from_min_size(pos2(o.x + pad, ext_top), vec2(side - 2.0 * pad, ext_h)),
            rail,
            chosen,
        );
        let speed = self.choice_speed_sample(&ctx);
        if card_h > 0.0 {
            self.choice_speed_card(
                ui,
                &p,
                &k,
                Rect::from_min_size(pos2(o.x + pad, card_top), vec2(side - 2.0 * pad, card_h)),
                speed,
            );
        }
        self.choice_footer(ui, &p, &k, o.x, foot_top, side, rail, chosen);

        // The library: the categories tree as navigation.
        if !rail {
            // i18n: MyNoX Choice sidebar: heading above All Downloads, Unfinished, Finished and the queues
            let lib = crate::i18n::t("Library").to_uppercase();
            text(&p, font(10.5, 1), o.x + 20.0, label_y, &lib, k.muted);
        } else {
            p.line_segment(
                [
                    at(o.x + 18.0, label_y - 4.0),
                    at(o.x + side - 18.0, label_y - 4.0),
                ],
                Stroke::new(1.0, k.faint),
            );
        }
        let nav = Rect::from_min_max(pos2(o.x, label_y + 8.0), pos2(o.x + side, ext_top - 8.0));
        self.choice_nav(ui, &p, &k, nav, rail);
    }

    // Which library rows show (on the rail: the top rows and the queues), and the height of each.
    fn choice_nav_rows(
        rows: &[(TreeRow<'static>, Option<View>, Branch)],
        rail: bool,
    ) -> Vec<(usize, f32)> {
        (0..rows.len())
            .filter(|i| {
                !rail
                    || rows[*i].0.level == 0
                    || rows[*i].1.is_some_and(|v| matches!(v, View::Queue(_)))
            })
            .map(|i| {
                (
                    i,
                    if rows[i].0.level == 0 || rail {
                        36.0
                    } else {
                        30.0
                    },
                )
            })
            .collect()
    }

    // All the library rows' height together.
    fn choice_nav_height(&self, rail: bool) -> f32 {
        Self::choice_nav_rows(&self.tree_rows(), rail)
            .iter()
            .map(|(_, h)| h)
            .sum()
    }

    // The library rows: All Downloads, Unfinished, Finished and the queues, with their counts.
    fn choice_nav(&mut self, ui: &mut Ui, p: &egui::Painter, k: &Ink, nav: Rect, rail: bool) {
        let ctx = ui.ctx().clone();
        let rows = self.tree_rows();
        let (free, kb_event) = self.tree_keyboard(&ctx, &rows);
        let keys = self.tree_keys && free;
        let shown = Self::choice_nav_rows(&rows, rail);
        let total: f32 = shown.iter().map(|(_, h)| h).sum();
        let room = nav.height().max(0.0);
        let mut scroll = self.choice.nav_scroll;
        if ui.rect_contains_pointer(winui::rect(nav)) {
            scroll -= ui.input(|i| i.smooth_scroll_delta.y);
        }
        scroll = scroll.clamp(0.0, (total - room).max(0.0));
        self.choice.nav_scroll = scroll;
        let clip = p.with_clip_rect(winui::rect(nav));
        let mut event = kb_event;
        let mut y = nav.top() - scroll;
        let pad = 12.0;
        for &(i, rh) in &shown {
            let (row, view, branch) = &rows[i];
            let indent = if rail || row.level == 0 { 0.0 } else { 18.0 };
            let rc = Rect::from_min_size(
                pos2(nav.left() + pad + indent, y + 1.0),
                vec2(nav.width() - 2.0 * pad - indent, rh - 2.0),
            );
            y += rh;
            if rc.bottom() < nav.top() || rc.top() > nav.bottom() {
                continue;
            }
            let hit = winui::rect(rc).intersect(winui::rect(nav));
            let resp = ui.interact(
                hit,
                egui::Id::unique(("choice-nav", i)),
                winui::pointer_only(),
            );
            let label = crate::i18n::t(&row.label).to_owned();
            winui::expose(
                &resp,
                egui::Role::TreeItem,
                &label,
                true,
                Some(row.selected),
            );
            let resp = if rail {
                resp.on_hover_text(&label)
            } else {
                resp
            };
            if row.selected {
                clip.rect_filled(winui::rect(rc), 9.0, k.accent.gamma_multiply(0.24));
                let bar = Rect::from_min_size(
                    pos2(rc.left() - 6.0, rc.top() + 8.0),
                    vec2(3.0, rc.height() - 16.0),
                );
                clip.rect_filled(winui::rect(bar), 2.0, k.accent);
            } else if resp.hovered() {
                clip.rect_filled(winui::rect(rc), 9.0, k.hover);
            }
            if keys && row.selected {
                winui::focus_ring_round(&clip, winui::rect(rc), 9.0);
            }
            let ink = if row.selected {
                Color32::WHITE
            } else if row.level == 0 {
                k.text
            } else {
                k.muted
            };
            let isz = if row.level == 0 || rail { 18.0 } else { 16.0 };
            let ix = if rail {
                rc.center().x - isz / 2.0
            } else {
                rc.left() + 10.0
            };
            glyph(
                ui,
                &clip,
                need_of(&row.icon),
                Rect::from_min_size(pos2(ix, rc.center().y - isz / 2.0), vec2(isz, isz)),
                ink,
            );
            if !rail {
                let f = if row.level == 0 {
                    font(13.0, 1)
                } else {
                    font(12.5, 0)
                };
                let right = rc.right() - if row.expander.is_some() { 26.0 } else { 8.0 };
                let mut count_w = 0.0;
                if let Some(n) = row.count {
                    let s = crate::i18n::integer(n as u64);
                    let pw = width(&clip, font(11.0, 1), &s) + 14.0;
                    count_w = pw + 6.0;
                    pill(
                        &clip,
                        right - pw,
                        rc.center().y,
                        &s,
                        if row.selected {
                            k.accent.gamma_multiply(0.55)
                        } else {
                            k.faint
                        },
                        ink,
                    );
                }
                let lx = rc.left() + 38.0;
                let shown_label =
                    winui::ellipsize(&clip, &f, &label, (right - count_w - lx).max(10.0));
                text(
                    &clip,
                    f,
                    lx,
                    mid(rc.top(), rc.height(), 13.0),
                    &shown_label,
                    ink,
                );
                if let Some(open) = row.expander {
                    chevron(&clip, pos2(rc.right() - 13.0, rc.center().y), open, k.muted);
                }
            }
            if resp.clicked() {
                let on_chevron = !rail
                    && row.expander.is_some()
                    && resp
                        .interact_pointer_pos()
                        .is_some_and(|q| winui::mx(q.x) >= rc.right() - 26.0);
                event = Some(if on_chevron || view.is_none() {
                    TreeEvent::Toggle(i)
                } else {
                    TreeEvent::Select(i)
                });
            }
            if resp.double_clicked() && row.expander.is_some() && *branch != Branch::Leaf {
                event = Some(TreeEvent::Toggle(i));
            }
            if resp.secondary_clicked()
                && let Some(q) = resp.interact_pointer_pos()
            {
                event = Some(TreeEvent::Context(i, q));
            }
        }
        // Fades where more rows are scrolled out of sight.
        let fade = |top: f32, down: bool| {
            let rc = Rect::from_min_size(pos2(nav.left(), top), vec2(nav.width(), 22.0));
            let (a, b) = if down {
                (Color32::TRANSPARENT, k.side)
            } else {
                (k.side, Color32::TRANSPARENT)
            };
            winui::fill_shaded(p, winui::rect(rc), 0.0, &[(0.0, a), (1.0, b)]);
        };
        if scroll > 0.5 {
            fade(nav.top(), false);
        }
        if scroll < total - room - 0.5 {
            fade(nav.bottom() - 22.0, true);
        }
        if let Some(ev) = event {
            self.tree_event(ev, &rows);
        }
    }

    // The browser extension's row: connected, or one click to add it.
    fn choice_extension(
        &mut self,
        ui: &mut Ui,
        p: &egui::Painter,
        k: &Ink,
        rc: Rect,
        rail: bool,
        chosen: &mut Option<(Cmd, Option<u64>)>,
    ) {
        // Read the extension's status file every 2 s, not every frame.
        let now = Instant::now();
        let seen = match self.choice.ext {
            Some((t, seen)) if now.duration_since(t) < Duration::from_secs(2) => seen,
            _ => {
                let seen = mynox_ipc::extension::seen();
                self.choice.ext = Some((now, seen));
                seen
            }
        };
        let label = if seen {
            // i18n: MyNoX Choice sidebar: the browser extension reached MyNoX
            crate::i18n::t("Browser connected")
        } else {
            // i18n: MyNoX Choice sidebar: one click to add the browser extension
            crate::i18n::t("Add to browser")
        };
        let r = ui.interact(
            winui::rect(rc),
            egui::Id::unique("choice-ext"),
            winui::pointer_only(),
        );
        winui::expose(&r, egui::Role::Button, label, true, None);
        // i18n: MyNoX Choice sidebar: tooltip of the browser extension's button
        let r = r.on_hover_text(crate::i18n::t("Catch downloads from your browser"));
        let (bg, edge) = if seen {
            (k.faint, Color32::TRANSPARENT)
        } else {
            (k.accent.gamma_multiply(0.16), k.accent.gamma_multiply(0.55))
        };
        p.rect_filled(
            winui::rect(rc),
            10.0,
            if r.hovered() {
                mix(bg, Color32::WHITE, 0.05)
            } else {
                bg
            },
        );
        p.rect_stroke(
            winui::rect(rc),
            10.0,
            Stroke::new(1.0, edge),
            StrokeKind::Inside,
        );
        let dot_x = if rail {
            rc.center().x
        } else {
            rc.left() + 16.0
        };
        let dot = if seen { k.good } else { k.warn };
        p.circle_filled(at(dot_x, rc.center().y), 7.0, dot.gamma_multiply(0.22));
        p.circle_filled(at(dot_x, rc.center().y), 4.0, dot);
        if !rail {
            text(
                p,
                font(12.5, 1),
                rc.left() + 30.0,
                mid(rc.top(), rc.height(), 12.5),
                label,
                k.text,
            );
            chevron(p, pos2(rc.right() - 14.0, rc.center().y), false, k.muted);
        }
        if r.clicked() {
            *chosen = Some((Cmd::BrowserExtension, None));
        }
    }

    // Keeps one sample a second of the speed of every download together; returns it now.
    fn choice_speed_sample(&mut self, ctx: &egui::Context) -> f64 {
        let running = self.items.iter().filter(|i| i.live.is_some());
        let speed: f64 = running.map(|i| i.rate()).sum();
        let now = Instant::now();
        let st = &mut self.choice;
        let due = st
            .sampled
            .map_or(1, |t| now.duration_since(t).as_secs() as usize);
        if due >= 1 {
            for _ in 1..due.min(SAMPLES) {
                st.speeds.push_back(0.0);
            }
            st.speeds.push_back(speed as f32);
            while st.speeds.len() > SAMPLES {
                st.speeds.pop_front();
            }
            st.sampled = Some(now);
        }
        if speed > 0.0 {
            ctx.request_repaint_after(Duration::from_secs(1));
        }
        speed
    }

    // The speed card: the speed now and a minute of it as a sparkline.
    fn choice_speed_card(&self, ui: &Ui, p: &egui::Painter, k: &Ink, rc: Rect, speed: f64) {
        let _ = ui;
        winui::fill_shaded(
            p,
            winui::rect(rc),
            12.0,
            &[(0.0, k.accent.gamma_multiply(0.22)), (1.0, k.faint)],
        );
        p.rect_stroke(
            winui::rect(rc),
            12.0,
            Stroke::new(1.0, k.faint),
            StrokeKind::Inside,
        );
        // i18n: MyNoX Choice sidebar: heading of the card that shows the download speed
        let head = crate::i18n::t("Download speed").to_uppercase();
        text(
            p,
            font(10.0, 1),
            rc.left() + 14.0,
            rc.top() + 20.0,
            &head,
            k.muted,
        );
        let running = self.items.iter().filter(|i| i.live.is_some()).count() as u64;
        let (big, sub) = if running > 0 {
            (
                crate::model::rate(speed),
                crate::i18n::tn(running, "{n} downloading", "{n} downloading"),
            )
        } else {
            // i18n: MyNoX Choice sidebar: the speed card while nothing downloads
            (crate::i18n::t("Idle").to_owned(), String::new())
        };
        let bw = text(
            p,
            font(18.0, 1),
            rc.left() + 14.0,
            rc.top() + 44.0,
            &big,
            k.text,
        );
        if !sub.is_empty() {
            let f = font(11.0, 0);
            let sw = width(p, f.clone(), &sub);
            if rc.left() + 14.0 + bw + 10.0 + sw <= rc.right() - 12.0 {
                text(p, f, rc.right() - 12.0 - sw, rc.top() + 44.0, &sub, k.muted);
            }
        }
        // The sparkline, the newest sample at the trailing edge.
        let spark = Rect::from_min_max(
            pos2(rc.left() + 12.0, rc.top() + 56.0),
            pos2(rc.right() - 12.0, rc.bottom() - 12.0),
        );
        let n = self.choice.speeds.len();
        let peak = self.choice.speeds.iter().copied().fold(0.0f32, f32::max);
        if n < 2 || peak <= 0.0 {
            let y = spark.bottom() - 1.0;
            p.line_segment(
                [at(spark.left(), y), at(spark.right(), y)],
                Stroke::new(1.0, k.faint),
            );
            return;
        }
        let step = spark.width() / (SAMPLES - 1) as f32;
        let x0 = spark.right() - step * (n - 1) as f32;
        let pts: Vec<Pos2> = self
            .choice
            .speeds
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let y = spark.bottom() - (v / peak) * (spark.height() - 2.0);
                at(x0 + step * i as f32, y)
            })
            .collect();
        // The area under the line, one band per step.
        let mut mesh = egui::Mesh::default();
        let fill_top = k.accent.gamma_multiply(0.35);
        let fill_bottom = k.accent.gamma_multiply(0.02);
        for w in pts.windows(2) {
            let base = mesh.vertices.len() as u32;
            mesh.colored_vertex(w[0], fill_top);
            mesh.colored_vertex(w[1], fill_top);
            mesh.colored_vertex(pos2(w[1].x, spark.bottom()), fill_bottom);
            mesh.colored_vertex(pos2(w[0].x, spark.bottom()), fill_bottom);
            mesh.add_triangle(base, base + 1, base + 2);
            mesh.add_triangle(base, base + 2, base + 3);
        }
        p.add(egui::Shape::mesh(mesh));
        p.add(egui::Shape::line(
            pts.clone(),
            Stroke::new(2.0, mix(k.accent, k.violet, 0.35)),
        ));
        if let Some(last) = pts.last() {
            p.circle_filled(*last, 3.5, Color32::WHITE);
            p.circle_filled(*last, 2.2, k.accent);
        }
    }

    // The small buttons at the bottom: Options, Scheduler, dark mode and folding the sidebar.
    #[allow(clippy::too_many_arguments)]
    fn choice_footer(
        &mut self,
        ui: &mut Ui,
        p: &egui::Painter,
        k: &Ink,
        x: f32,
        top: f32,
        side: f32,
        rail: bool,
        chosen: &mut Option<(Cmd, Option<u64>)>,
    ) {
        p.line_segment(
            [at(x + 14.0, top), at(x + side - 14.0, top)],
            Stroke::new(1.0, k.faint),
        );
        // View menu's own words (already translated): Always dark, Show / Hide categories.
        let mode_tip = "Always dark";
        let side_tip = if rail {
            "Show &categories"
        } else {
            "Hide &categories"
        };
        let buttons: [(&str, Cmd, &str); 4] = [
            ("tb-options", Cmd::Options, "Options"),
            ("tb-scheduler", Cmd::Scheduler, "Scheduler"),
            ("", Cmd::ToggleAlwaysDark, mode_tip),
            ("", Cmd::ToggleCategories, side_tip),
        ];
        // On the rail they stand in a column above the foot; open, in a row.
        for (i, (need, cmd, tip)) in buttons.iter().enumerate() {
            let rc = if rail {
                Rect::from_min_size(
                    pos2(x + (side - 36.0) / 2.0, top + 6.0 + i as f32 * 40.0),
                    vec2(36.0, 36.0),
                )
            } else {
                let gap = (side - 28.0 - 4.0 * 36.0) / 3.0;
                Rect::from_min_size(
                    pos2(x + 14.0 + i as f32 * (36.0 + gap), top + 6.0),
                    vec2(36.0, 36.0),
                )
            };
            let r = ui.interact(
                winui::rect(rc),
                egui::Id::unique(("choice-foot", i)),
                winui::pointer_only(),
            );
            let tip = crate::i18n::t(tip).replace('&', "");
            let tip = tip.as_str();
            winui::expose(&r, egui::Role::Button, tip, true, None);
            let r = r.on_hover_text(tip);
            if r.hovered() {
                p.rect_filled(winui::rect(rc), 10.0, k.hover);
            }
            let c = rc.center();
            match i {
                0 | 1 => glyph(
                    ui,
                    p,
                    need,
                    Rect::from_center_size(c, vec2(18.0, 18.0)),
                    k.text,
                ),
                2 => {
                    // A moon: a disc with a disc of the sidebar's colour bitten out of it.
                    let cc = at(c.x, c.y);
                    p.circle_filled(cc, 7.5, k.text);
                    let bg = if r.hovered() {
                        mix(k.side, Color32::WHITE, 0.06)
                    } else {
                        k.side
                    };
                    p.circle_filled(at(c.x + 4.0, c.y - 3.5), 6.5, bg);
                }
                _ => {
                    // Two chevrons pointing where the sidebar goes.
                    let s = Stroke::new(1.7, k.text);
                    let d = if rail { 1.0 } else { -1.0 };
                    for off in [-3.0f32, 3.0] {
                        let cx = c.x + off;
                        p.line_segment([at(cx - 2.5 * d, c.y - 5.0), at(cx + 2.5 * d, c.y)], s);
                        p.line_segment([at(cx + 2.5 * d, c.y), at(cx - 2.5 * d, c.y + 5.0)], s);
                    }
                }
            }
            if r.clicked() {
                *chosen = Some((*cmd, None));
            }
        }
    }

    // The header: the view's name and count, what is going on, and Find.
    fn choice_header(
        &mut self,
        ui: &mut Ui,
        x: f32,
        top: f32,
        w: f32,
        chosen: &mut Option<(Cmd, Option<u64>)>,
    ) {
        let p = ui.painter().clone();
        let c = crate::theme::colors();
        let title = self
            .tree_rows()
            .into_iter()
            .find(|(_, v, _)| *v == Some(self.view))
            .map(|(r, ..)| crate::i18n::t(&r.label).to_owned())
            .unwrap_or_default();
        let search_w = if w >= 640.0 {
            260.0
        } else if w >= 460.0 {
            180.0
        } else {
            40.0
        };
        let left = x + 26.0;
        let right = x + w - 26.0;
        let tf = font(21.0, 1);
        let room = (right - search_w - 24.0 - 60.0 - left).max(40.0);
        let shown = winui::ellipsize(&p, &tf, &title, room);
        let tw = text(&p, tf, left, top + 36.0, &shown, c.text);
        let n = self.view_ids().len() as u64;
        pill(
            &p,
            left + tw + 10.0,
            top + 29.0,
            &crate::i18n::integer(n),
            c.accent.gamma_multiply(0.14),
            c.accent,
        );
        let sub = self.activity().unwrap_or_else(|| {
            // i18n: MyNoX Choice header: under the view's name while no download runs
            crate::i18n::t("Nothing is downloading right now").to_owned()
        });
        let sf = font(12.0, 0);
        let sub = winui::ellipsize(&p, &sf, &sub, (right - search_w - 24.0 - left).max(40.0));
        text(&p, sf, left, top + 56.0, &sub, c.muted);

        // Find: a search box that opens Find (Ctrl+F).
        let rc = Rect::from_min_size(pos2(right - search_w, top + 18.0), vec2(search_w, 36.0));
        let r = ui.interact(
            winui::rect(rc),
            egui::Id::unique("choice-find"),
            winui::pointer_only(),
        );
        // i18n: MyNoX Choice header: the search box's placeholder (opens Find)
        let hint = crate::i18n::t("Find a download");
        winui::expose(&r, egui::Role::Button, hint, true, None);
        let r = if search_w < 100.0 {
            r.on_hover_text(hint)
        } else {
            r
        };
        p.rect_filled(winui::rect(rc), 10.0, c.field);
        p.rect_stroke(
            winui::rect(rc),
            10.0,
            Stroke::new(
                1.0,
                if r.hovered() {
                    c.accent.gamma_multiply(0.7)
                } else {
                    c.list_frame
                },
            ),
            StrokeKind::Inside,
        );
        // A magnifier.
        let mx = rc.left()
            + if search_w < 100.0 {
                search_w / 2.0 - 1.0
            } else {
                18.0
            };
        let my = rc.center().y - 1.0;
        let ink = Stroke::new(1.6, c.muted);
        p.circle_stroke(at(mx, my), 5.5, ink);
        p.line_segment([at(mx + 4.0, my + 4.0), at(mx + 8.0, my + 8.0)], ink);
        if search_w >= 100.0 {
            let kbd = "Ctrl+F";
            let kf = font(10.5, 1);
            let kw = width(&p, kf.clone(), kbd) + 12.0;
            let kr = Rect::from_min_size(
                pos2(rc.right() - 8.0 - kw, rc.center().y - 10.0),
                vec2(kw, 20.0),
            );
            p.rect_filled(winui::rect(kr), 6.0, c.header);
            p.rect_stroke(
                winui::rect(kr),
                6.0,
                Stroke::new(1.0, c.list_frame),
                StrokeKind::Inside,
            );
            text(
                &p,
                kf,
                kr.left() + 6.0,
                mid(kr.top(), 20.0, 10.5),
                kbd,
                c.muted,
            );
            let hf = font(12.5, 0);
            let hint = winui::ellipsize(
                &p,
                &hf,
                hint,
                (kr.left() - 8.0 - (rc.left() + 32.0)).max(10.0),
            );
            text(
                &p,
                hf,
                rc.left() + 32.0,
                mid(rc.top(), rc.height(), 12.5),
                &hint,
                c.muted,
            );
        }
        if r.clicked() {
            *chosen = Some((Cmd::Find, None));
        }
    }

    // The action bar: the toolbar's commands as pills, queues' split, and the limiter's switch.
    fn choice_actions(
        &mut self,
        ui: &mut Ui,
        x: f32,
        top: f32,
        w: f32,
        chosen: &mut Option<(Cmd, Option<u64>)>,
    ) {
        let p = ui.painter().clone();
        let c = crate::theme::colors();
        let picked = self.chosen(None);
        let running = picked.iter().any(|i| i.live.is_some());
        let resumable = picked.iter().any(|i| i.can_start());
        let deletable = picked.iter().any(|i| i.live.is_none());
        let any_running = self.items.iter().any(|i| i.live.is_some());
        let any_complete = self.items.iter().any(|i| i.status == Status::Complete);
        let main_running = self.queues.first().is_some_and(|q| q.running);
        let has_main = !self.queues.is_empty();
        let buttons: [Action; 7] = [
            ("tb-resume", "Resume", resumable, Cmd::Resume, None, 0),
            ("tb-stop", "Stop", running, Cmd::Stop, None, 0),
            (
                "tb-stop-all",
                "Stop All",
                any_running,
                Cmd::StopAll,
                None,
                0,
            ),
            ("tb-delete", "Delete", deletable, Cmd::Remove, None, 1),
            (
                "tb-delete-completed",
                "Delete Completed",
                any_complete,
                Cmd::DeleteCompleted,
                None,
                1,
            ),
            (
                "tb-start-queue",
                "Start Queue",
                has_main && !main_running,
                Cmd::StartQueue(0),
                Some(true),
                2,
            ),
            (
                "tb-stop-queue",
                "Stop Queue",
                main_running,
                Cmd::StopQueue(0),
                Some(false),
                2,
            ),
        ];
        let lf = font(12.5, 1);
        let left = x + 18.0;
        // The limiter's switch on the trailing side, when there is room.
        // The Downloads menu's own words (already translated).
        let lim_label = crate::i18n::tc("menu", "Speed &Limiter").replace('&', "");
        let lim_label = lim_label.as_str();
        let lim_w = width(&p, font(12.0, 1), lim_label) + 58.0;
        let label_w: Vec<f32> = buttons
            .iter()
            .map(|(_, l, ..)| width(&p, lf.clone(), crate::i18n::tc("toolbar", l)) + 42.0)
            .collect();
        let bar_w = |labeled: &[bool]| -> f32 {
            buttons
                .iter()
                .enumerate()
                .map(|(i, b)| if labeled[i] { label_w[i] } else { 40.0 } + if b.4.is_some() { 24.0 } else { 0.0 })
                .sum::<f32>()
                + 6.0 * 6.0
                + 2.0 * 16.0
        };
        let avail = w - 36.0;
        let show_lim = avail >= bar_w(&[false; 7]) + lim_w + 16.0;
        let room = avail - if show_lim { lim_w + 16.0 } else { 0.0 };
        // The labels give way in this order: Delete Completed, the queues', Stop All, Delete.
        let mut labeled = [true; 7];
        for drop in [4usize, 6, 5, 2, 3, 1, 0] {
            if bar_w(&labeled) <= room {
                break;
            }
            labeled[drop] = false;
        }
        let mut bx = left;
        let mut group = 0u8;
        for (i, (need, label, enabled, cmd, drop, g)) in buttons.iter().enumerate() {
            if *g != group {
                group = *g;
                bx += 8.0;
                p.line_segment(
                    [at(bx, top + 8.0), at(bx, top + ACTIONS_H - 8.0)],
                    Stroke::new(1.0, c.list_frame),
                );
                bx += 8.0;
            }
            let full = crate::i18n::tc("toolbar", label);
            let labels = labeled[i];
            let bw = if labels { label_w[i] } else { 40.0 };
            let dw = if drop.is_some() { 24.0 } else { 0.0 };
            let rc = Rect::from_min_size(pos2(bx, top), vec2(bw + dw, ACTIONS_H));
            let main = Rect::from_min_size(rc.min, vec2(bw, ACTIONS_H));
            let r = ui.interact(
                winui::rect(main),
                egui::Id::unique(("choice-act", i)),
                winui::pointer_only(),
            );
            winui::expose(&r, egui::Role::Button, full, *enabled, None);
            let r = if labels { r } else { r.on_hover_text(full) };
            let hot = *enabled && r.hovered();
            let face = if hot && r.is_pointer_button_down_on() {
                mix(c.field, c.accent, 0.16)
            } else if hot {
                mix(c.field, c.accent, 0.08)
            } else {
                c.field
            };
            p.rect_filled(winui::rect(rc), 10.0, face);
            p.rect_stroke(
                winui::rect(rc),
                10.0,
                Stroke::new(
                    1.0,
                    if hot {
                        c.accent.gamma_multiply(0.55)
                    } else {
                        c.list_frame
                    },
                ),
                StrokeKind::Inside,
            );
            let ink = if *enabled { c.accent } else { c.text_disabled };
            let ix = if labels {
                main.left() + 12.0
            } else {
                main.center().x - 9.0
            };
            glyph(
                ui,
                &p,
                need,
                Rect::from_min_size(pos2(ix, rc.center().y - 9.0), vec2(18.0, 18.0)),
                ink,
            );
            if labels {
                text(
                    &p,
                    lf.clone(),
                    main.left() + 34.0,
                    mid(rc.top(), rc.height(), 12.5),
                    full,
                    if *enabled { c.text } else { c.text_disabled },
                );
            }
            if r.clicked() && *enabled {
                *chosen = Some((*cmd, None));
            }
            if let Some(start) = drop {
                // The arrow: every queue, also while the main one's button is greyed.
                let arrow = Rect::from_min_max(pos2(main.right(), rc.top()), rc.max);
                let ar = ui.interact(
                    winui::rect(arrow),
                    egui::Id::unique(("choice-act-drop", i)),
                    winui::pointer_only(),
                );
                if ar.hovered() {
                    p.rect_filled(
                        winui::rect(arrow.shrink(3.0)),
                        7.0,
                        mix(c.field, c.accent, 0.10),
                    );
                }
                p.line_segment(
                    [
                        at(arrow.left(), rc.top() + 9.0),
                        at(arrow.left(), rc.bottom() - 9.0),
                    ],
                    Stroke::new(1.0, c.list_frame),
                );
                chevron(&p, pos2(arrow.center().x, arrow.center().y), true, c.muted);
                if ar.clicked() {
                    self.open_popup(Popup::QueueDrop(*start), at(rc.left(), rc.bottom() + 4.0));
                }
            }
            bx += bw + dw + 6.0;
        }
        if show_lim {
            let on = self.settings.speed_limit_on;
            let rc = Rect::from_min_size(pos2(x + w - 18.0 - lim_w, top), vec2(lim_w, ACTIONS_H));
            if rc.left() > bx + 8.0 {
                let r = ui.interact(
                    winui::rect(rc),
                    egui::Id::unique("choice-limiter"),
                    winui::pointer_only(),
                );
                winui::expose(&r, egui::Role::CheckBox, lim_label, true, Some(on));
                if r.hovered() {
                    p.rect_filled(winui::rect(rc), 10.0, mix(c.window, c.accent, 0.06));
                }
                text(
                    &p,
                    font(12.0, 1),
                    rc.left() + 10.0,
                    mid(rc.top(), rc.height(), 12.0),
                    lim_label,
                    c.text,
                );
                // The switch: a track and a knob.
                let tr = Rect::from_min_size(
                    pos2(rc.right() - 44.0, rc.center().y - 10.0),
                    vec2(36.0, 20.0),
                );
                p.rect_filled(
                    winui::rect(tr),
                    10.0,
                    if on { c.accent } else { c.list_frame },
                );
                let kx = if on {
                    tr.right() - 10.0
                } else {
                    tr.left() + 10.0
                };
                p.circle_filled(at(kx, tr.center().y), 7.5, Color32::WHITE);
                if r.clicked() {
                    *chosen = Some((if on { Cmd::LimiterOff } else { Cmd::LimiterOn }, None));
                }
                if r.secondary_clicked() {
                    *chosen = Some((Cmd::LimiterSettings, None));
                }
            }
        }
    }

    // The status bar under the card: how many shown and selected, and what runs.
    fn choice_status(&self, ui: &Ui, x: f32, top: f32, w: f32) {
        let p = ui.painter();
        let c = crate::theme::colors();
        let n = self.shown_ids().len() as u64;
        let mut left = crate::i18n::tn(n, "{n} download", "{n} downloads");
        let selected = self.selection.ids().len() as u64;
        if selected > 1 {
            left += "  \u{00b7}  ";
            left += &crate::i18n::tn(selected, "{n} selected", "{n} selected");
        }
        let f = font(11.5, 0);
        let base = mid(top, STATUS_H, 11.5);
        winui::fit(
            ui,
            p,
            x + 26.0,
            base,
            &left,
            c.muted,
            x + w / 2.0,
            "choice-strip-left",
        );
        if let Some(right) = self.activity() {
            let rw = width(p, f.clone(), &right);
            let rx = x + w - 26.0 - rw;
            p.circle_filled(at(rx - 10.0, top + STATUS_H / 2.0), 3.5, c.success);
            text(p, f, rx, base, &right, c.muted);
        }
    }
}
