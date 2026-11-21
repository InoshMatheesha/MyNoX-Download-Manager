//! MyNoX Island's main window, technical glass: the Dynamic Island of live activity at the top,
//! the view's name and the speed set large in type, the scope switch and the category and queue
//! chips, the list on a glass card, and a floating dock of round buttons. A child module of
//! app.rs (as choice.rs is): it runs the same commands (`Cmd`), popups and views.

use std::collections::{HashSet, VecDeque};
use std::time::{Duration, Instant};

use egui::{Rect, Stroke, Ui, pos2, vec2};

use super::{App, Cmd, Popup, View};
use crate::glass::{self, Ink, at, mid, mono, text, text_end, width};
use crate::model::{Category, Item, Status};
use crate::winui;

// The header (the view's name, the island, the speed), the filter bar and the dock's band.
const HEAD_H: f32 = 98.0;
const FILTER_H: f32 = 42.0;
const FOOT_H: f32 = 76.0;
// A minute and a half of speed samples, one a second.
const SAMPLES: usize = 90;
// How long the island tells of a download that just finished.
const FLASH: Duration = Duration::from_secs(6);

// A dock button: icon, label (the toolbar's words), enabled, command, its drop-down, group.
type DockButton = (&'static str, &'static str, bool, Cmd, Option<Popup>, u8);

// What the layout keeps between frames.
#[derive(Default)]
pub(super) struct State {
    speeds: VecDeque<f32>,
    sampled: Option<Instant>,
    // The finished downloads seen last, to tell of the next one (counted first: rebuilt only
    // when that count changes).
    complete: Option<(usize, HashSet<u64>)>,
    flash: Option<(u64, String, Instant)>,
    // How far the chip strip is scrolled sideways.
    chip_scroll: f32,
}

// What the island shows.
#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Idle,
    Live,
    Flash,
}

// A one-colour icon of the theme's set in `ink`, inside `rc` (English).
fn glyph(ui: &Ui, need: &str, rc: Rect, ink: egui::Color32) {
    let px = rc.width() * ui.ctx().pixels_per_point();
    if let Some((src, tint)) = crate::themeicons::source(need, px, ink, true) {
        egui::Image::new(src)
            .tint(tint)
            .paint_at(ui, winui::rect(rc).intersect(ui.painter().clip_rect()));
    }
}

// A small chevron pointing down, centred on `c` (English).
fn chevron(p: &egui::Painter, cx: f32, cy: f32, ink: egui::Color32) {
    let s = Stroke::new(1.5, ink);
    p.line_segment([at(cx - 3.5, cy - 1.5), at(cx, cy + 2.0)], s);
    p.line_segment([at(cx, cy + 2.0), at(cx + 3.5, cy - 1.5)], s);
}

impl App {
    /// The backdrop under the menu bar and everything else (drawn first).
    pub(super) fn island_backdrop(&self, ui: &Ui, o: egui::Pos2, w: f32, h: f32) {
        let t = ui.input(|i| i.time);
        glass::backdrop(ui.painter(), Rect::from_min_size(o, vec2(w, h)), t);
    }

    /// Paints the header, the island, the filter bar, the card and the dock; returns the list's
    /// rect inside the card.
    pub(super) fn island_chrome(
        &mut self,
        ui: &mut Ui,
        o: egui::Pos2,
        w: f32,
        h: f32,
        chosen: &mut Option<(Cmd, Option<u64>)>,
    ) -> Rect {
        let top = o.y + crate::chrome::MENU_H;
        let speed = self.island_watch(ui.ctx());
        self.island_header(ui, o.x, top, w, speed);
        self.island_filters(ui, o.x, top + HEAD_H, w, chosen);
        let card = Rect::from_min_max(
            pos2(o.x + 20.0, top + HEAD_H + FILTER_H + 8.0),
            pos2(o.x + w - 20.0, o.y + h - FOOT_H),
        );
        glass::card(ui.painter(), card, 18.0, &Ink::now());
        self.island_dock(ui, o.x, o.y + h - FOOT_H, w, chosen);
        // Last: opened, it grows down over the header.
        self.island_island(ui, o.x, top, w, chosen);
        card.shrink2(vec2(8.0, 8.0))
    }

    // One sample a second of every download's speed together, and the download that just
    // finished; returns the speed now.
    fn island_watch(&mut self, ctx: &egui::Context) -> f64 {
        let speed: f64 = self
            .items
            .iter()
            .filter(|i| i.live.is_some())
            .map(|i| i.rate())
            .sum();
        let now = Instant::now();
        let st = &mut self.island;
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
        let count = self
            .items
            .iter()
            .filter(|i| i.status == Status::Complete)
            .count();
        let changed = st.complete.as_ref().is_none_or(|(n, _)| *n != count);
        if changed {
            let done: HashSet<u64> = self
                .items
                .iter()
                .filter(|i| i.status == Status::Complete)
                .map(|i| i.id)
                .collect();
            // The first look only learns what is there; a later one tells of what is new.
            if let Some((_, before)) = &st.complete
                && let Some(new) = self
                    .items
                    .iter()
                    .rev()
                    .find(|i| done.contains(&i.id) && !before.contains(&i.id))
            {
                st.flash = Some((new.id, new.file_name.clone(), now));
            }
            st.complete = Some((count, done));
        }
        if let Some((.., t)) = st.flash {
            let gone = now.duration_since(t);
            if gone >= FLASH {
                st.flash = None;
            } else {
                ctx.request_repaint_after(FLASH - gone);
            }
        }
        speed
    }

    // The view's name, as the island's header shows it.
    fn island_title(&self) -> String {
        let cat = |c: Category| crate::i18n::t(&c.label()).to_owned();
        match self.view {
            View::All => crate::i18n::t("All Downloads").to_owned(),
            View::Cat(c) | View::Unfinished(Some(c)) | View::Finished(Some(c)) => cat(c),
            View::Unfinished(None) => crate::i18n::t("Unfinished").to_owned(),
            View::Finished(None) => crate::i18n::t("Finished").to_owned(),
            View::Queue(q) => self
                .queues
                .get(q)
                .map(|q| q.name.clone())
                .unwrap_or_default(),
        }
    }

    // How wide the island may grow in a window `w` wide.
    fn island_max_w(w: f32) -> f32 {
        (w - 2.0 * 250.0).clamp(150.0, 420.0)
    }

    // The header: the scope and count over the view's name; the speed over its sparkline.
    fn island_header(&self, ui: &Ui, x: f32, top: f32, w: f32, speed: f64) {
        let p = ui.painter();
        let c = crate::theme::colors();
        let k = Ink::now();
        let (left, right) = (x + 30.0, x + w - 30.0);
        let island_left = x + w / 2.0 - Self::island_max_w(w) / 2.0;
        let room = (island_left - 20.0 - left).max(60.0);
        let n = self.view_ids().len() as u64;
        // The scope over a category's name ("Unfinished / 3 downloads"), "Library" otherwise.
        let scope = match self.view {
            View::Unfinished(Some(_)) => crate::i18n::t("Unfinished"),
            View::Finished(Some(_)) => crate::i18n::t("Finished"),
            View::Cat(_) => crate::i18n::t("All Downloads"),
            // i18n: MyNoX Choice sidebar: heading above All Downloads, Unfinished, Finished and the queues
            _ => crate::i18n::t("Library"),
        };
        let eyebrow = format!(
            "{}  /  {}",
            scope,
            crate::i18n::tn(n, "{n} download", "{n} downloads")
        );
        glass::eyebrow(p, left, top + 34.0, &eyebrow, room, &k);
        let tf = glass::bold(28.0);
        let title = winui::ellipsize(p, &tf, &self.island_title(), room);
        text(p, tf, left, top + 68.0, &title, c.text);

        // The speed on the trailing side, large, over a minute and a half of it.
        let running = self.items.iter().filter(|i| i.live.is_some()).count();
        let block =
            190.0f32.min((right - (x + w / 2.0 + Self::island_max_w(w) / 2.0) - 16.0).max(90.0));
        let head = crate::i18n::t("Download speed");
        let hf = mono(10.0);
        let head = winui::ellipsize(p, &hf, &head.to_uppercase(), block);
        text_end(p, hf, right, top + 34.0, &head, k.muted);
        let (value, ink) = if running > 0 {
            (crate::model::rate(speed), k.accent)
        } else {
            // i18n: MyNoX Choice sidebar: the speed card while nothing downloads
            (crate::i18n::t("Idle").to_owned(), c.text)
        };
        let vf = glass::fit(p, mono, 22.0, 15.0, &value, block);
        let value = winui::ellipsize(p, &vf, &value, block);
        text_end(p, vf, right, top + 62.0, &value, ink);
        let samples: Vec<f32> = self.island.speeds.iter().copied().collect();
        glass::sparkline(
            p,
            Rect::from_min_max(pos2(right - block, top + 72.0), pos2(right, top + 90.0)),
            &samples,
            SAMPLES,
            k.accent,
            &k,
        );
    }

    // The Dynamic Island: a dark capsule that tells what is going on. Idle, a small pill;
    // while downloads run, the first one's ring, name and speed (the pointer on it opens it
    // further: how many run, the time left, a bar); a download that just finished, for a few
    // seconds. A click opens that download's window (or the finished file); idle, Add URL.
    fn island_island(
        &mut self,
        ui: &mut Ui,
        x: f32,
        top: f32,
        w: f32,
        chosen: &mut Option<(Cmd, Option<u64>)>,
    ) {
        let ctx = ui.ctx().clone();
        let p = ui.painter().clone();
        let k = Ink::now();
        let lead: Option<&Item> = self.items.iter().find(|i| i.live.is_some());
        let running = self.items.iter().filter(|i| i.live.is_some()).count();
        let flash = self.island.flash.clone();
        let mode = if flash.is_some() {
            Mode::Flash
        } else if lead.is_some() {
            Mode::Live
        } else {
            Mode::Idle
        };
        let key = egui::Id::unique("island-capsule");
        let was_hot = ctx.data(|d| d.get_temp::<bool>(key.with("hot")).unwrap_or(false));
        let open = was_hot && mode == Mode::Live;
        let max_w = Self::island_max_w(w);
        let want_w = match mode {
            Mode::Idle => 150.0f32.min(max_w),
            _ => max_w,
        };
        let iw = ctx.animate_value_with_time(key.with("w"), want_w, 0.34);
        let ih = ctx.animate_value_with_time(key.with("h"), if open { 78.0 } else { 36.0 }, 0.26);
        let rc = Rect::from_min_size(pos2(x + w / 2.0 - iw / 2.0, top + 10.0), vec2(iw, ih));
        let r = ui.interact(winui::rect(rc), key, winui::pointer_only());
        ctx.data_mut(|d| d.insert_temp(key.with("hot"), r.hovered()));
        let what = match mode {
            Mode::Idle => crate::i18n::tc("toolbar", "Add URL").to_owned(),
            Mode::Live => self.activity().unwrap_or_default(),
            Mode::Flash => flash.as_ref().map(|f| f.1.clone()).unwrap_or_default(),
        };
        winui::expose(&r, egui::Role::Button, &what, true, None);
        let radius = 18.0f32.min(ih / 2.0);
        p.rect_filled(
            winui::rect(rc.expand(2.0).translate(vec2(0.0, 6.0))),
            radius + 2.0,
            k.shadow.gamma_multiply(0.7),
        );
        p.rect_filled(winui::rect(rc), radius, k.island);
        p.rect_stroke(
            winui::rect(rc),
            radius,
            Stroke::new(1.0, k.on_island.gamma_multiply(0.07)),
            egui::StrokeKind::Inside,
        );
        let clip = p.with_clip_rect(winui::rect(rc).intersect(p.clip_rect()));
        let row = rc.top() + 18.0;
        // The lens at the trailing end, as a phone's island has its camera.
        let lens_x = rc.right() - 18.0;
        clip.circle_filled(at(lens_x, row), 6.0, k.on_island.gamma_multiply(0.06));
        clip.circle_filled(at(lens_x, row), 3.0, k.on_island.gamma_multiply(0.10));
        clip.circle_filled(
            at(lens_x - 1.2, row - 1.2),
            1.0,
            k.on_island.gamma_multiply(0.35),
        );
        let t = ui.input(|i| i.time);
        match mode {
            Mode::Idle => {
                // A slow breath of the signal colour.
                let a = 0.55 + 0.45 * ((t * 1.6).sin() as f32 * 0.5 + 0.5);
                clip.circle_filled(
                    at(rc.left() + 20.0, row),
                    3.5,
                    k.island_signal.gamma_multiply(a),
                );
                let f = mono(11.0);
                // i18n: MyNoX Island: the Dynamic Island while nothing downloads (ready for the next download)
                let s = crate::i18n::t("Ready").to_uppercase();
                let s = winui::ellipsize(&clip, &f, &s, iw - 64.0);
                text(
                    &clip,
                    f,
                    rc.left() + 32.0,
                    mid(rc.top(), 36.0, 11.0),
                    &s,
                    k.island_muted,
                );
                if r.hovered() {
                    ctx.request_repaint_after(Duration::from_millis(50));
                }
            }
            Mode::Live => {
                let Some(lead) = lead else { return };
                let frac = lead.fraction().map(|f| f as f32);
                glass::ring(
                    &clip,
                    pos2(rc.left() + 20.0, row),
                    7.0,
                    2.6,
                    frac,
                    t,
                    k.on_island.gamma_multiply(0.14),
                    k.island_signal,
                );
                if frac.is_none() {
                    ctx.request_repaint_after(Duration::from_millis(33));
                }
                let sf = mono(12.0);
                let speed = crate::model::rate(lead.rate());
                let sw = text_end(
                    &clip,
                    sf,
                    lens_x - 16.0,
                    mid(rc.top(), 36.0, 12.0),
                    &speed,
                    k.island_signal,
                );
                let nf = glass::display(12.5);
                let name = winui::ellipsize(
                    &clip,
                    &nf,
                    &lead.file_name,
                    (lens_x - 16.0 - sw - 14.0 - (rc.left() + 36.0)).max(20.0),
                );
                text(
                    &clip,
                    nf,
                    rc.left() + 36.0,
                    mid(rc.top(), 36.0, 12.5),
                    &name,
                    k.on_island,
                );
                // Opened: how many run, all together, and the time left; a bar under them.
                let shown = ((ih - 36.0) / 42.0).clamp(0.0, 1.0);
                if shown > 0.05 {
                    let mut line = self.activity().unwrap_or_default();
                    let left = lead.time_left();
                    if !left.is_empty() {
                        line = format!("{line}  \u{00b7}  {left}");
                    }
                    let lf = mono(10.5);
                    let line = winui::ellipsize(&clip, &lf, &line, iw - 40.0);
                    text(
                        &clip,
                        lf,
                        rc.left() + 20.0,
                        rc.top() + 50.0,
                        &line,
                        k.island_muted.gamma_multiply(shown),
                    );
                    let bar = Rect::from_min_size(
                        pos2(rc.left() + 20.0, rc.top() + 62.0),
                        vec2(iw - 40.0, 4.0),
                    );
                    clip.rect_filled(
                        winui::rect(bar),
                        2.0,
                        k.on_island.gamma_multiply(0.12 * shown),
                    );
                    if let Some(f) = frac {
                        let done = Rect::from_min_size(bar.min, vec2(bar.width() * f, 4.0));
                        clip.rect_filled(
                            winui::rect(done),
                            2.0,
                            k.island_signal.gamma_multiply(shown),
                        );
                    }
                }
                if running > 1 {
                    // The others: a count by the ring.
                    let f = mono(9.5);
                    let s = format!("+{}", running - 1);
                    let bw = width(&clip, f.clone(), &s) + 8.0;
                    let badge =
                        Rect::from_min_size(pos2(rc.left() + 24.0, row - 15.0), vec2(bw, 13.0));
                    clip.rect_filled(winui::rect(badge), 6.5, k.island_signal);
                    text(
                        &clip,
                        f,
                        badge.left() + 4.0,
                        badge.bottom() - 3.0,
                        &s,
                        k.island,
                    );
                }
            }
            Mode::Flash => {
                let Some((_, name, _)) = &flash else { return };
                // A tick in a green disc.
                let cx = rc.left() + 20.0;
                clip.circle_filled(at(cx, row), 8.0, k.island_good);
                let s = Stroke::new(1.8, k.island);
                clip.line_segment([at(cx - 3.5, row), at(cx - 1.0, row + 2.5)], s);
                clip.line_segment([at(cx - 1.0, row + 2.5), at(cx + 3.8, row - 2.8)], s);
                let df = mono(11.0);
                // i18n: a download's status (state it is in, not a command): finished
                let done = crate::i18n::t("Complete").to_uppercase();
                let dw = text_end(
                    &clip,
                    df,
                    lens_x - 16.0,
                    mid(rc.top(), 36.0, 11.0),
                    &done,
                    k.island_good,
                );
                let nf = glass::display(12.5);
                let shown = winui::ellipsize(
                    &clip,
                    &nf,
                    name,
                    (lens_x - 16.0 - dw - 14.0 - (rc.left() + 36.0)).max(20.0),
                );
                text(
                    &clip,
                    nf,
                    rc.left() + 36.0,
                    mid(rc.top(), 36.0, 12.5),
                    &shown,
                    k.on_island,
                );
            }
        }
        if r.clicked() {
            match mode {
                Mode::Idle => *chosen = Some((Cmd::AddUrl, None)),
                Mode::Live => {
                    if let Some(id) = lead.map(|l| l.id) {
                        self.open_item(id);
                    }
                }
                Mode::Flash => {
                    if let Some((id, ..)) = flash {
                        self.island.flash = None;
                        self.open_item(id);
                    }
                }
            }
        }
    }

    // The filter bar: All / Unfinished / Finished, the categories and the queues as chips, and
    // Find. A chip's right-click is its node's menu, as in the Categories tree.
    fn island_filters(
        &mut self,
        ui: &mut Ui,
        x: f32,
        top: f32,
        w: f32,
        chosen: &mut Option<(Cmd, Option<u64>)>,
    ) {
        let p = ui.painter().clone();
        let c = crate::theme::colors();
        let k = Ink::now();
        let (left, right) = (x + 24.0, x + w - 24.0);
        let scope = match self.view {
            View::All | View::Cat(_) => Some(0),
            View::Unfinished(_) => Some(1),
            View::Finished(_) => Some(2),
            View::Queue(_) => None,
        };
        let cat = match self.view {
            View::Cat(c) => Some(c),
            View::Unfinished(c) | View::Finished(c) => c,
            _ => None,
        };
        let in_scope = |s: usize, c: Option<Category>| match (s, c) {
            (1, c) => View::Unfinished(c),
            (2, c) => View::Finished(c),
            (_, Some(c)) => View::Cat(c),
            (_, None) => View::All,
        };
        let labels = [
            crate::i18n::t("All Downloads"),
            crate::i18n::t("Unfinished"),
            crate::i18n::t("Finished"),
        ];
        let (picked, seg_w) =
            glass::segmented(ui, left, top + 4.0, 34.0, &labels, scope, "island-scope");
        if let Some(s) = picked {
            // The category stays picked across the scopes.
            self.view = in_scope(s, cat);
        }

        // Find on the trailing side: a search box, or its magnifier alone in a narrow window.
        let fw = if w >= 980.0 {
            230.0
        } else if w >= 760.0 {
            180.0
        } else {
            36.0
        };
        let find = Rect::from_min_size(pos2(right - fw, top + 5.0), vec2(fw, 32.0));
        let r = ui.interact(
            winui::rect(find),
            egui::Id::unique("island-find"),
            winui::pointer_only(),
        );
        // i18n: MyNoX Choice header: the search box's placeholder (opens Find)
        let hint = crate::i18n::t("Find a download");
        winui::expose(&r, egui::Role::Button, hint, true, None);
        p.rect_filled(
            winui::rect(find),
            16.0,
            if r.hovered() { k.glass_hot } else { k.glass },
        );
        p.rect_stroke(
            winui::rect(find),
            16.0,
            Stroke::new(
                1.0,
                if r.hovered() {
                    k.accent.gamma_multiply(0.6)
                } else {
                    k.edge
                },
            ),
            egui::StrokeKind::Inside,
        );
        let mx = find.left() + if fw < 100.0 { fw / 2.0 - 1.0 } else { 17.0 };
        let my = find.center().y - 1.0;
        let s = Stroke::new(1.6, k.muted);
        p.circle_stroke(at(mx, my), 5.0, s);
        p.line_segment([at(mx + 3.7, my + 3.7), at(mx + 7.5, my + 7.5)], s);
        if fw >= 100.0 {
            // The shortcut where there is room for it beside the words.
            let kf = mono(10.0);
            let kbd = "Ctrl+F";
            let kw = if fw >= 200.0 {
                text_end(
                    &p,
                    kf,
                    find.right() - 14.0,
                    mid(find.top(), 32.0, 10.0),
                    kbd,
                    k.muted,
                )
            } else {
                -12.0
            };
            let hf = glass::display(12.5);
            let hint = winui::ellipsize(&p, &hf, hint, fw - 32.0 - kw - 26.0);
            text(
                &p,
                hf,
                find.left() + 30.0,
                mid(find.top(), 32.0, 12.5),
                &hint,
                k.muted,
            );
        } else {
            let _ = r.clone().on_hover_text(hint);
        }
        if r.clicked() {
            *chosen = Some((Cmd::Find, None));
        }

        // The chips, while the categories show (View → Show categories): a strip that scrolls
        // sideways under the wheel when they don't all fit, faded where more are out of sight.
        if !self.show_categories {
            return;
        }
        let strip = Rect::from_min_max(
            pos2(left + seg_w + 12.0, top + 2.0),
            pos2(find.left() - 10.0, top + FILTER_H - 2.0),
        );
        if strip.width() < 40.0 {
            return;
        }
        let cats: Vec<(Category, String)> = Category::named()
            .into_iter()
            .map(|ct| (ct, crate::i18n::t(&ct.label()).to_owned()))
            .collect();
        let queue_w: Vec<f32> = self
            .queues
            .iter()
            .map(|q| glass::chip_w(&p, &q.name) + 16.0)
            .collect();
        let total = cats
            .iter()
            .map(|(_, l)| glass::chip_w(&p, l) + 2.0)
            .sum::<f32>()
            + if queue_w.is_empty() { 0.0 } else { 18.0 }
            + queue_w.iter().map(|w| w + 2.0).sum::<f32>()
            + 4.0;
        let most = (total - strip.width()).max(0.0);
        let mut scroll = self.island.chip_scroll;
        if most > 0.0 && ui.rect_contains_pointer(winui::rect(strip)) {
            let d = ui.input(|i| i.smooth_scroll_delta);
            scroll -= if d.x != 0.0 { d.x } else { d.y };
        }
        scroll = scroll.clamp(0.0, most);
        self.island.chip_scroll = scroll;
        let clip = winui::rect(strip).intersect(ui.clip_rect());
        let mut sui = ui.new_child(egui::UiBuilder::new().max_rect(clip));
        sui.set_clip_rect(clip);
        let sp = sui.painter().clone();
        let mut cx = strip.left() + 4.0 - scroll;
        let cy = top + 7.0;
        let mut menu: Option<(View, egui::Pos2)> = None;
        for (i, (ct, label)) in cats.iter().enumerate() {
            let cw = glass::chip_w(&p, label);
            let rc = Rect::from_min_size(pos2(cx, cy), vec2(cw, 28.0));
            cx += cw + 2.0;
            if rc.right() < strip.left() || rc.left() > strip.right() {
                continue;
            }
            let on = scope.is_some() && cat == Some(*ct);
            let r = glass::chip(&sui, rc, ("island-cat", i), label, on, false);
            if r.clicked() {
                let s = scope.unwrap_or(0);
                self.view = in_scope(s, if on { None } else { Some(*ct) });
            }
            if r.secondary_clicked()
                && let Some(q) = r.interact_pointer_pos()
            {
                menu = Some((in_scope(scope.unwrap_or(0), Some(*ct)), q));
            }
        }
        if !queue_w.is_empty() {
            cx += 8.0;
            sp.line_segment(
                [at(cx, top + 12.0), at(cx, top + 30.0)],
                Stroke::new(1.0, c.list_frame),
            );
            cx += 10.0;
        }
        for (q, cw) in queue_w.iter().enumerate() {
            let rc = Rect::from_min_size(pos2(cx, cy), vec2(*cw, 28.0));
            cx += cw + 2.0;
            if rc.right() < strip.left() || rc.left() > strip.right() {
                continue;
            }
            let label = self.queues[q].name.clone();
            let on = self.view == View::Queue(q);
            let r = glass::chip(&sui, rc, ("island-queue", q), "", on, false);
            winui::expose(&r, egui::Role::CheckBox, &label, true, Some(on));
            // The queue's running dot, then its name.
            let running = self.queues[q].running;
            let ink = if on { k.text } else { k.muted };
            let dx = rc.left() + if on { 26.0 } else { 14.0 };
            sp.circle_stroke(
                at(dx, rc.center().y),
                3.5,
                Stroke::new(1.4, if running { c.success } else { ink }),
            );
            if running {
                sp.circle_filled(at(dx, rc.center().y), 2.0, c.success);
            }
            text(
                &sp,
                glass::display(12.5),
                dx + 9.0,
                mid(rc.top(), 28.0, 12.5),
                &label,
                ink,
            );
            if r.clicked() {
                self.view = View::Queue(q);
            }
            if r.secondary_clicked()
                && let Some(q2) = r.interact_pointer_pos()
            {
                menu = Some((View::Queue(q), q2));
            }
        }
        // Fades over the ends where chips are scrolled out of sight.
        let fade = 26.0;
        if scroll > 0.5 {
            glass::fade_side(
                &sp,
                Rect::from_min_size(strip.min, vec2(fade, strip.height())),
                c.window,
                true,
            );
        }
        if scroll < most - 0.5 {
            glass::fade_side(
                &sp,
                Rect::from_min_size(
                    pos2(strip.right() - fade, strip.top()),
                    vec2(fade, strip.height()),
                ),
                c.window,
                false,
            );
        }
        if let Some((v, q)) = menu {
            self.open_popup(Popup::Node(Some(v)), q);
        }
    }

    // The dock: the toolbar's commands as round glass buttons in a floating capsule, Add URL
    // the large signal-coloured one; what shows and what is picked at its leading side, the
    // speed limiter's switch at its trailing side.
    fn island_dock(
        &mut self,
        ui: &mut Ui,
        x: f32,
        top: f32,
        w: f32,
        chosen: &mut Option<(Cmd, Option<u64>)>,
    ) {
        let p = ui.painter().clone();
        let c = crate::theme::colors();
        let k = Ink::now();
        let picked = self.chosen(None);
        let running = picked.iter().any(|i| i.live.is_some());
        let resumable = picked.iter().any(|i| i.can_start());
        let deletable = picked.iter().any(|i| i.live.is_none());
        let any_running = self.items.iter().any(|i| i.live.is_some());
        let any_complete = self.items.iter().any(|i| i.status == Status::Complete);
        let main_running = self.queues.first().is_some_and(|q| q.running);
        let has_main = !self.queues.is_empty();
        let buttons: [DockButton; 12] = [
            ("tb-add-url", "Add URL", true, Cmd::AddUrl, None, 0),
            (
                "tb-add-torrent",
                "Add Torrent",
                true,
                Cmd::OpenTorrentFile,
                Some(Popup::TorrentDrop),
                0,
            ),
            ("tb-resume", "Resume", resumable, Cmd::Resume, None, 1),
            ("tb-stop", "Stop", running, Cmd::Stop, None, 1),
            (
                "tb-stop-all",
                "Stop All",
                any_running,
                Cmd::StopAll,
                None,
                1,
            ),
            ("tb-delete", "Delete", deletable, Cmd::Remove, None, 2),
            (
                "tb-delete-completed",
                "Delete Completed",
                any_complete,
                Cmd::DeleteCompleted,
                None,
                2,
            ),
            (
                "tb-start-queue",
                "Start Queue",
                has_main && !main_running,
                Cmd::StartQueue(0),
                Some(Popup::QueueDrop(true)),
                3,
            ),
            (
                "tb-stop-queue",
                "Stop Queue",
                main_running,
                Cmd::StopQueue(0),
                Some(Popup::QueueDrop(false)),
                3,
            ),
            ("tb-scheduler", "Scheduler", true, Cmd::Scheduler, None, 4),
            ("tb-options", "Options", true, Cmd::Options, None, 4),
            ("", "Always dark", true, Cmd::ToggleAlwaysDark, None, 4),
        ];
        // The size that fits: 44 px buttons, smaller in a narrow window.
        let dock_w = |s: f32| -> f32 {
            let mut total = 12.0 * 2.0;
            let mut group = 0u8;
            for (i, b) in buttons.iter().enumerate() {
                if b.5 != group {
                    group = b.5;
                    total += 17.0;
                }
                total += if i == 0 { s + 12.0 } else { s };
                total += if b.4.is_some() { 18.0 } else { 0.0 };
                total += 4.0;
            }
            total - 4.0
        };
        let s = [40.0f32, 36.0, 32.0]
            .into_iter()
            .find(|s| dock_w(*s) <= w - 40.0)
            .unwrap_or(32.0);
        let dw = dock_w(s);
        let dh = s + 18.0;
        let dock = Rect::from_min_size(
            pos2(x + w / 2.0 - dw / 2.0, top + (FOOT_H - dh) / 2.0 + 2.0),
            vec2(dw, dh),
        );
        glass::panel(&p, dock, dh / 2.0, &k);
        let mut bx = dock.left() + 12.0;
        let mut group = 0u8;
        let cy = dock.center().y;
        let mut hint: Option<(Rect, String)> = None;
        for (i, (need, label, enabled, cmd, drop, g)) in buttons.iter().enumerate() {
            if *g != group {
                group = *g;
                bx += 6.0;
                p.line_segment(
                    [at(bx, cy - s * 0.32), at(bx, cy + s * 0.32)],
                    Stroke::new(1.0, k.edge),
                );
                bx += 11.0;
            }
            let primary = i == 0;
            let d = if primary { s + 12.0 } else { s };
            let rc = Rect::from_min_size(pos2(bx, cy - d / 2.0), vec2(d, d));
            let words = if *label == "Always dark" {
                crate::i18n::t(label).to_owned()
            } else {
                crate::i18n::tc("toolbar", label).to_owned()
            };
            let r = ui.interact(
                winui::rect(rc),
                egui::Id::unique(("island-dock", i)),
                winui::pointer_only(),
            );
            winui::expose(&r, egui::Role::Button, &words, *enabled, None);
            let hot = *enabled && r.hovered();
            let down = hot && r.is_pointer_button_down_on();
            // Pressed, a button sinks a little.
            let face = rc.shrink(if down { 1.5 } else { 0.0 });
            let ink = if primary {
                let fill = if down {
                    c.button_accent_pressed
                } else if hot {
                    c.button_accent_hot
                } else {
                    c.button_accent
                };
                p.circle_filled(
                    at(face.center().x, face.center().y + 3.0),
                    d / 2.0,
                    c.button_accent.gamma_multiply(0.25),
                );
                p.circle_filled(
                    at(face.center().x, face.center().y),
                    face.width() / 2.0,
                    fill,
                );
                c.button_accent_text
            } else {
                if hot {
                    p.circle_filled(
                        at(face.center().x, face.center().y),
                        face.width() / 2.0,
                        k.glass_hot,
                    );
                    p.circle_stroke(
                        at(face.center().x, face.center().y),
                        face.width() / 2.0 - 0.5,
                        Stroke::new(1.0, k.edge),
                    );
                }
                if *enabled { k.text } else { k.disabled }
            };
            let gs = (s * 0.42).round();
            let gc = face.center();
            if primary {
                // A plus, drawn: the one button that needs no icon to be read.
                let st = Stroke::new(2.4, ink);
                let a = gs * 0.5;
                p.line_segment([at(gc.x - a, gc.y), at(gc.x + a, gc.y)], st);
                p.line_segment([at(gc.x, gc.y - a), at(gc.x, gc.y + a)], st);
            } else if need.is_empty() {
                // A moon: a disc with the dock's glass bitten out of it.
                let mc = at(gc.x, gc.y);
                p.circle_filled(mc, gs * 0.42, ink);
                let behind = glass::over(c.window, k.glass);
                let behind = if hot {
                    glass::over(behind, k.glass_hot)
                } else {
                    behind
                };
                p.circle_filled(at(gc.x + gs * 0.22, gc.y - gs * 0.2), gs * 0.36, behind);
            } else {
                glyph(ui, need, Rect::from_center_size(gc, vec2(gs, gs)), ink);
            }
            if r.hovered() {
                hint = Some((winui::rect(rc), words.clone()));
            }
            if r.clicked() && *enabled {
                *chosen = Some((*cmd, None));
            }
            bx += d;
            if let Some(popup) = drop {
                // The drop-down: a narrow capsule after the button (every queue, also while the
                // main one's button is greyed).
                let arrow =
                    Rect::from_min_size(pos2(bx - 2.0, cy - s * 0.36), vec2(18.0, s * 0.72));
                let ar = ui.interact(
                    winui::rect(arrow),
                    egui::Id::unique(("island-dock-drop", i)),
                    winui::pointer_only(),
                );
                // i18n: MyNoX Choice sidebar: tooltip of the arrow beside Add Torrent (magnet links, RSS feeds...)
                let more = if i == 1 {
                    crate::i18n::t("More torrent options").to_owned()
                } else {
                    words.clone()
                };
                winui::expose(&ar, egui::Role::Button, &more, true, None);
                if ar.hovered() {
                    p.rect_filled(winui::rect(arrow), 9.0, k.glass_hot);
                    hint = Some((winui::rect(arrow), more));
                }
                chevron(&p, arrow.center().x, arrow.center().y, k.muted);
                if ar.clicked() {
                    self.open_popup(*popup, at(arrow.left(), dock.top() - 4.0));
                }
                bx += 18.0;
            }
            bx += 4.0;
        }
        if let Some((rc, label)) = hint {
            glass::tip(ui, rc, &label);
        }

        // What shows (and is picked), at the leading side when there is room.
        let base = mid(dock.top(), dock.height(), 10.5);
        let n = self.shown_ids().len() as u64;
        let mut left = crate::i18n::tn(n, "{n} download", "{n} downloads");
        let selected = self.selection.ids().len() as u64;
        if selected > 1 {
            left += "  \u{00b7}  ";
            left += &crate::i18n::tn(selected, "{n} selected", "{n} selected");
        }
        let lf = mono(10.5);
        let room = dock.left() - 20.0 - (x + 30.0);
        if room > 80.0 {
            let shown = winui::ellipsize(&p, &lf, &left.to_uppercase(), room);
            text(&p, lf, x + 30.0, base, &shown, k.muted);
        }

        // The speed limiter's switch, at the trailing side when there is room.
        let lim = crate::i18n::tc("menu", "Speed &Limiter").replace('&', "");
        let lf = glass::display(12.5);
        let lw = width(&p, lf.clone(), &lim) + 54.0;
        let lr = Rect::from_min_size(
            pos2(x + w - 30.0 - lw, dock.center().y - 16.0),
            vec2(lw, 32.0),
        );
        if lr.left() > dock.right() + 16.0 {
            let on = self.settings.speed_limit_on;
            let r = ui.interact(
                winui::rect(lr),
                egui::Id::unique("island-limiter"),
                winui::pointer_only(),
            );
            winui::expose(&r, egui::Role::CheckBox, &lim, true, Some(on));
            if r.hovered() {
                p.rect_filled(winui::rect(lr), 16.0, k.glass);
            }
            text(
                &p,
                lf,
                lr.left() + 10.0,
                mid(lr.top(), 32.0, 12.5),
                &lim,
                k.text,
            );
            glass::switch(
                &p,
                Rect::from_min_size(
                    pos2(lr.right() - 44.0, lr.center().y - 10.0),
                    vec2(36.0, 20.0),
                ),
                on,
                &k,
            );
            if r.clicked() {
                *chosen = Some((if on { Cmd::LimiterOff } else { Cmd::LimiterOn }, None));
            }
            if r.secondary_clicked() {
                *chosen = Some((Cmd::LimiterSettings, None));
            }
        }
    }
}
