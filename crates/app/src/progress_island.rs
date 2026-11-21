//! MyNoX Island's download window, technical glass: a small Dynamic Island with the state, a
//! progress ring, the file's name and its speed set large, four readings, and two rows of chips
//! that act at once (a speed limit, what happens when it is done) over Pause and Cancel. Its
//! details open below: the connections (a torrent's own tabs), the full Speed Limiter or Options
//! on completion, the classic pages unchanged. A child module of progress.rs: it uses that
//! window's state and pages.

use egui::{Color32, Pos2, Rect, Stroke, Ui, pos2, vec2};
use mynox_engine::Snapshot;

use super::{Action, Progress, Tab, connections, status_text, torrent_drop};
use crate::glass::{self, Ink, at, mid, mono, text, text_mid, width};
use crate::model::{self, Item, Status, size3};
use crate::winui;

/// The body without and with its details.
const COMPACT_H: f32 = 404.0;
const DETAILS_H: f32 = 620.0;
/// Where the details' pages are laid out: their own origin this far down.
const PAGES_DY: f32 = 410.0;
/// The connections' strip and table (segment_bar's y 249 lands at 456).
const CONNECTIONS_DY: f32 = 207.0;
/// The speed limits offered as chips, in KB/s (None: off).
const LIMITS: [Option<u64>; 4] = [None, Some(256), Some(1024), Some(4096)];

/// The window's body size: its details open or not.
pub(super) fn size(details: bool, item: &Item) -> [f32; 2] {
    if details {
        [548.0, DETAILS_H + torrent_drop(item)]
    } else {
        [548.0, COMPACT_H]
    }
}

// A rate as the chips show it: whole KB/s, or MB/s from 1 MB/s (the units' own words).
fn limit_label(kb: Option<u64>) -> String {
    match kb {
        None => crate::i18n::t("Off").to_owned(),
        Some(kb) if kb >= 1024 && kb % 1024 == 0 => crate::i18n::fmt(
            crate::i18n::tc("unit", "{n} MB/sec"),
            &[("n", &crate::i18n::integer(kb / 1024))],
        ),
        Some(kb) => crate::i18n::fmt(
            crate::i18n::tc("unit", "{n} KB/sec"),
            &[("n", &crate::i18n::integer(kb))],
        ),
    }
}

// The server's name in an address (its host), for the line over the file's name.
fn host(item: &Item) -> String {
    item.url.host_str().unwrap_or_default().to_owned()
}

// A state's colour inside the island (always dark: the dark variant's colours).
fn island_ink(item: &Item, k: &Ink) -> Color32 {
    let d = &crate::theme::current().dark;
    match item.status {
        Status::Downloading => k.island_signal,
        Status::Complete => k.island_good,
        Status::Error => d.error,
        Status::Queued => d.warning,
        Status::Stopped => k.island_muted,
    }
}

// The connections' parts of the file as a rail: each one from its start to where it has come,
// in the signal colour on the track, with a notch where each starts.
fn island_segments(p: &egui::Painter, o: Pos2, item: &Item, snap: Option<&Snapshot>, k: &Ink) {
    let c = crate::theme::colors();
    let track = Rect::from_min_size(pos2(o.x + 24.0, o.y + 456.0), vec2(500.0, 10.0));
    p.rect_filled(winui::rect(track), 5.0, c.progress_track);
    let Some(total) = item.size.filter(|t| *t > 0) else {
        return;
    };
    let x_of = |v: u64| track.left() + (v as f64 / total as f64).min(1.0) as f32 * track.width();
    let parts: Vec<(u64, u64)> = match snap {
        Some(s) if !s.segments.is_empty() => {
            s.segments.iter().map(|g| (g.start, g.cursor)).collect()
        }
        _ => vec![(0, item.downloaded)],
    };
    let ink = if item.status == Status::Complete {
        c.success
    } else {
        k.accent
    };
    for (start, cursor) in &parts {
        let a = x_of(*start);
        let b = x_of(*cursor).max(a + 2.0).min(track.right());
        let run = Rect::from_min_max(pos2(a, track.top()), pos2(b, track.bottom()));
        p.rect_filled(winui::rect(run), 5.0, ink);
    }
    for (start, _) in parts.iter().filter(|(s, _)| *s > 0) {
        let x = x_of(*start);
        p.line_segment(
            [at(x, track.top() - 3.0), at(x, track.bottom() + 3.0)],
            Stroke::new(1.0, k.muted),
        );
    }
}

impl Progress {
    /// The download window's body in MyNoX Island.
    pub(super) fn island_body(
        &mut self,
        ui: &mut Ui,
        o: Pos2,
        item: &Item,
        snap: Option<&Snapshot>,
        frac: Option<f64>,
    ) -> Action {
        let p = ui.painter().clone();
        let c = crate::theme::colors();
        let k = Ink::now();
        let [bw, bh] = size(self.details, item);
        let t = ui.input(|i| i.time);
        glass::backdrop(&p, Rect::from_min_size(o, vec2(bw, bh)), t);
        let x = |v: f32| o.x + v;
        let y = |v: f32| o.y + v;
        let (left, right) = (x(24.0), x(524.0));

        // The state in a small island.
        let status = status_text(item, snap);
        let sf = glass::display(12.0);
        let shown = winui::ellipsize(&p, &sf, &status, 300.0);
        let iw = (width(&p, sf.clone(), &shown) + 62.0).max(150.0);
        let cap = Rect::from_min_size(pos2(x(274.0) - iw / 2.0, y(14.0)), vec2(iw, 30.0));
        p.rect_filled(
            winui::rect(cap.translate(vec2(0.0, 4.0))),
            15.0,
            k.shadow.gamma_multiply(0.6),
        );
        p.rect_filled(winui::rect(cap), 15.0, k.island);
        let ink = island_ink(item, &k);
        p.circle_filled(
            at(cap.left() + 16.0, cap.center().y),
            5.5,
            ink.gamma_multiply(0.25),
        );
        p.circle_filled(at(cap.left() + 16.0, cap.center().y), 3.0, ink);
        let drawn = winui::text_font(
            &p,
            sf,
            cap.left() + 28.0,
            mid(cap.top(), 30.0, 12.0),
            &shown,
            if item.status == Status::Error {
                ink
            } else {
                k.on_island
            },
        );
        winui::copyable(ui, drawn, "status", &status, shown != status);
        let lens = cap.right() - 15.0;
        p.circle_filled(
            at(lens, cap.center().y),
            5.0,
            k.on_island.gamma_multiply(0.06),
        );
        p.circle_filled(
            at(lens, cap.center().y),
            2.4,
            k.on_island.gamma_multiply(0.12),
        );
        // The hide button where the system draws the title bar (Linux, macOS; on Windows it is
        // in ours).
        let mut action = Action::None;
        if self.tray
            && !cfg!(windows)
            && winui::to_tray_button(
                ui,
                winui::r(o, 500.0, 18.0, 22.0, 20.0),
                "progress-hide",
                crate::i18n::mark("Hide to the notification area"),
            )
            .clicked()
        {
            action = Action::Hide;
        }

        // The ring: the share done, and what it is a share of.
        let ring_ink = match item.status {
            Status::Complete => c.success,
            Status::Error => c.error,
            Status::Downloading => c.progress_bar,
            _ => c.muted,
        };
        let rc = pos2(x(96.0), y(142.0));
        let running = item.status == Status::Downloading;
        glass::ring(
            &p,
            rc,
            58.0,
            9.0,
            frac.map(|f| f as f32).or((!running).then_some(0.0)),
            t,
            c.progress_track,
            ring_ink,
        );
        if frac.is_none() && running {
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(33));
        }
        match (frac, item.size) {
            (Some(f), Some(size)) => {
                text_mid(
                    &p,
                    mono(24.0),
                    rc.x,
                    rc.y + 6.0,
                    &crate::i18n::percent(f, 1),
                    c.text,
                );
                text_mid(&p, mono(10.0), rc.x, rc.y + 24.0, &size3(size), k.muted);
            }
            _ => text_mid(
                &p,
                mono(15.0),
                rc.x,
                rc.y + 5.0,
                &size3(item.downloaded),
                c.text,
            ),
        }

        // The file: its type and server, its name, its speed and the time left.
        let cx = x(188.0);
        let ext = std::path::Path::new(&item.file_name)
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.chars().take(6).collect::<String>())
            .unwrap_or_default();
        let line = [ext, crate::i18n::ltr(&host(item))]
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("  \u{00b7}  ");
        glass::eyebrow(&p, cx, y(88.0), &line, right - cx, &k);
        let nf = glass::fit(&p, glass::bold, 19.0, 15.0, &item.file_name, right - cx);
        let name = winui::ellipsize(&p, &nf, &item.file_name, right - cx);
        let drawn = winui::text_font(&p, nf, cx, y(114.0), &name, c.text);
        winui::copyable(ui, drawn, "name", &item.file_name, name != item.file_name);
        let (speed, ink) = if running {
            (model::rate(item.rate()), k.accent)
        } else {
            ("\u{2014}".to_owned(), k.muted)
        };
        let vf = glass::fit(&p, mono, 28.0, 18.0, &speed, right - cx);
        let speed = winui::ellipsize(&p, &vf, &speed, right - cx);
        text(&p, vf, cx, y(160.0), &speed, ink);
        let left_now = item.time_left();
        if !left_now.is_empty() {
            let lf = glass::display(12.5);
            let lw = text(&p, lf, cx, y(186.0), crate::i18n::t("Time left"), k.muted);
            text(&p, mono(12.5), cx + lw + 10.0, y(186.0), &left_now, c.text);
        }

        // Four readings, set in type alone: no boxes.
        let dash = || "\u{2014}".to_owned();
        let readings = [
            (crate::i18n::t("Downloaded"), size3(item.downloaded)),
            (
                crate::i18n::t("File size"),
                item.size.map_or_else(dash, size3),
            ),
            (
                crate::i18n::t("Resume capability"),
                match item.resumable {
                    Some(true) => crate::i18n::t("Yes").to_owned(),
                    Some(false) => crate::i18n::t("No").to_owned(),
                    None => dash(),
                },
            ),
        ];
        let col = 110.0;
        for (i, (label, value)) in readings.iter().enumerate() {
            let cx = left + col * i as f32;
            glass::eyebrow(&p, cx, y(228.0), label, col - 12.0, &k);
            let vf = glass::fit(&p, mono, 13.0, 10.0, value, col - 12.0);
            let v = winui::ellipsize(&p, &vf, value, col - 12.0);
            let drawn = winui::text_font(&p, vf, cx, y(248.0), &v, c.text);
            winui::copyable(ui, drawn, ("reading", i), value, &v != value);
        }
        // Where its server is (#140): a file download's.
        let sx = left + col * 3.0;
        if item.torrent.is_none() && item.video.is_none() && item.gallery.is_none() {
            glass::eyebrow(&p, sx, y(228.0), crate::i18n::t("Server"), right - sx, &k);
            let seen = snap
                .and_then(|s| s.server.as_ref())
                .map(crate::whereis::Seen::from)
                .or_else(|| item.server.clone());
            // Its round trip runs on after the place: kept inside the window.
            let room = winui::rect(Rect::from_min_max(
                pos2(sx, y(230.0)),
                pos2(right, y(256.0)),
            ))
            .intersect(ui.clip_rect());
            let mut cell = ui.new_child(egui::UiBuilder::new().max_rect(room));
            cell.set_clip_rect(room);
            crate::whereis::line(
                &cell,
                sx,
                y(248.0),
                right,
                seen.as_ref(),
                &format!("progress-{}", item.id),
            );
        }

        // The chips: a speed limit at once, and what to do when it is done.
        self.island_limits(ui, o, item, &k);
        self.island_when_done(ui, o, &k);

        // Pause or Resume, Cancel, and the details.
        let can_resume = matches!(item.status, Status::Stopped | Status::Error);
        let (label, enabled) = if running {
            // i18n: verb: pause the download
            (crate::i18n::mark("Pause"), true)
        } else if can_resume {
            // i18n: verb: continue the download
            (crate::i18n::mark("Resume"), true)
        } else {
            (crate::i18n::mark("Pause"), false)
        };
        let row = y(354.0);
        if glass::button(
            ui,
            Rect::from_min_size(pos2(left, row), vec2(140.0, 36.0)),
            crate::i18n::t(label),
            "progress-pause",
            glass::Kind::Primary,
            enabled,
        )
        .clicked()
            && enabled
        {
            action = if running {
                Action::Pause
            } else {
                Action::Resume
            };
        }
        if glass::button(
            ui,
            Rect::from_min_size(pos2(left + 150.0, row), vec2(112.0, 36.0)),
            crate::i18n::t("Cancel"),
            "progress-cancel",
            glass::Kind::Glass,
            true,
        )
        .clicked()
        {
            action = Action::Cancel;
        }
        // i18n: MyNoX Island's download window: the button that opens and closes the window's details (its connections, the speed limiter, the options on completion), and the switch's first part (a download's connections, a torrent's files, peers and trackers)
        let words = crate::i18n::t("Details");
        let df = glass::display(13.0);
        let dw = width(&p, df.clone(), words) + 46.0;
        let dr = Rect::from_min_size(pos2(right - dw, row), vec2(dw, 36.0));
        let r = ui.interact(
            winui::rect(dr),
            egui::Id::unique("progress-island-details"),
            winui::pointer_only(),
        );
        winui::expose(&r, egui::Role::Button, words, true, Some(self.details));
        if r.hovered() {
            p.rect_filled(winui::rect(dr), 18.0, k.glass_hot);
        }
        text(
            &p,
            df,
            dr.left() + 14.0,
            mid(row, 36.0, 13.0),
            words,
            c.text,
        );
        // A chevron: down to open, up to close.
        let (vx, vy) = (dr.right() - 18.0, dr.center().y);
        let s = Stroke::new(1.6, k.muted);
        let d = if self.details { -1.0 } else { 1.0 };
        p.line_segment([at(vx - 4.0, vy - 2.0 * d), at(vx, vy + 2.0 * d)], s);
        p.line_segment([at(vx, vy + 2.0 * d), at(vx + 4.0, vy - 2.0 * d)], s);
        if r.clicked() {
            self.details = !self.details;
        }

        if self.details {
            self.island_details(ui, o, item, snap, &k);
        }
        action
    }

    // The speed limit as chips: Off and three rates; a rate set elsewhere shows as its own chip.
    // A chip applies at once, as the Speed Limiter tab's box does once typed, and is
    // remembered for the file when that tab's "Remember" is ticked.
    fn island_limits(&mut self, ui: &mut Ui, o: Pos2, item: &Item, k: &Ink) {
        self.init_limit(item);
        let p = ui.painter().clone();
        let top = o.y + 268.0;
        glass::eyebrow(
            &p,
            o.x + 24.0,
            mid(top, 28.0, 10.0),
            crate::i18n::t("Speed Limiter"),
            120.0,
            k,
        );
        let now: Option<u64> = self
            .limiter
            .then(|| self.limit_kb.parse::<u64>().ok())
            .flatten()
            .filter(|kb| *kb > 0);
        let mut chips: Vec<Option<u64>> = LIMITS.to_vec();
        if now.is_some() && !chips.contains(&now) {
            chips.push(now);
        }
        let mut cx = o.x + 150.0;
        for (i, kb) in chips.iter().enumerate() {
            let label = limit_label(*kb);
            let cw = glass::chip_w(&p, &label);
            if cx + cw > o.x + 524.0 {
                break;
            }
            let rc = Rect::from_min_size(pos2(cx, top), vec2(cw, 28.0));
            if glass::chip(ui, rc, ("progress-limit", i), &label, now == *kb, false).clicked() {
                match kb {
                    Some(kb) => {
                        self.limiter = true;
                        self.limit_kb = kb.to_string();
                    }
                    None => self.limiter = false,
                }
                self.pending_limit = Some((kb.map(|kb| kb * 1024), self.remember));
            }
            cx += cw + 4.0;
        }
    }

    // What happens when it is done: the complete dialog, or (with it off) exiting MyNoX or
    // turning the computer off as the third chip names it. Picking either of those turns the
    // dialog off, as they apply only without it (Options on completion greys them while it is
    // on); picking the dialog leaves them as they were, greyed.
    fn island_when_done(&mut self, ui: &mut Ui, o: Pos2, k: &Ink) {
        let p = ui.painter().clone();
        let top = o.y + 306.0;
        // i18n: MyNoX Island's download window: label before the chips that say what happens when the download finishes
        let label = crate::i18n::t("When done");
        glass::eyebrow(&p, o.x + 24.0, mid(top, 28.0, 10.0), label, 120.0, k);
        let dialog = self.show_complete;
        let chips = [
            // i18n: MyNoX Island's download window: a chip: show the "Download complete" window when the download finishes
            (crate::i18n::t("Show dialog").to_owned(), dialog, false),
            // i18n: MyNoX Island's download window: a chip: close MyNoX Download Manager when the download finishes; keep "MyNoX" as written
            (
                crate::i18n::t("Exit MyNoX").to_owned(),
                !dialog && self.exit,
                dialog,
            ),
            (
                crate::i18n::t(crate::queue::TurnOff::LABELS[self.turn_off]).to_owned(),
                !dialog && self.turn_off_on,
                dialog,
            ),
        ];
        let mut cx = o.x + 150.0;
        for (i, (label, on, dim)) in chips.iter().enumerate() {
            let cw = glass::chip_w(&p, label);
            let rc = Rect::from_min_size(pos2(cx, top), vec2(cw, 28.0));
            if glass::chip(ui, rc, ("progress-done", i), label, *on, *dim).clicked() {
                match i {
                    0 => self.show_complete = !self.show_complete,
                    1 => {
                        self.exit = !*on;
                        if self.exit {
                            self.show_complete = false;
                        }
                    }
                    _ => {
                        self.turn_off_on = !*on;
                        if self.turn_off_on {
                            self.show_complete = false;
                        }
                    }
                }
            }
            cx += cw + 4.0;
        }
    }

    // The details: a switch over the connections (a torrent's tabs), the Speed Limiter's page
    // and Options on completion's, laid out as they are in the other themes.
    fn island_details(
        &mut self,
        ui: &mut Ui,
        o: Pos2,
        item: &Item,
        snap: Option<&Snapshot>,
        k: &Ink,
    ) {
        let p = ui.painter().clone();
        let tabs = [Tab::Status, Tab::Limiter, Tab::Completion];
        let labels = [
            crate::i18n::t("Details"),
            crate::i18n::t("Speed Limiter"),
            crate::i18n::t("Options on completion"),
        ];
        let current = tabs.iter().position(|t| *t == self.tab).unwrap_or(0);
        // A hairline across, then the switch.
        p.line_segment(
            [
                at(o.x + 24.0, o.y + COMPACT_H),
                at(o.x + 524.0, o.y + COMPACT_H),
            ],
            Stroke::new(1.0, k.edge),
        );
        let (picked, _) = glass::segmented(
            ui,
            o.x + 24.0,
            o.y + COMPACT_H + 10.0,
            32.0,
            &labels,
            Some(current),
            "progress-island-pane",
        );
        if let Some(i) = picked {
            self.tab = tabs[i];
        }
        match self.tab {
            Tab::Status if item.torrent.is_some() => {
                self.torrent_details(
                    ui,
                    &p,
                    o + vec2(0.0, CONNECTIONS_DY + torrent_drop(item)),
                    item,
                );
            }
            Tab::Status => {
                island_segments(&p, o, item, snap, k);
                connections(ui, &p, o + vec2(0.0, CONNECTIONS_DY), snap);
            }
            Tab::Limiter => {
                self.limiter_page(ui, o + vec2(0.0, PAGES_DY), item);
            }
            Tab::Completion => {
                self.completion_page(ui, o + vec2(0.0, PAGES_DY), item);
            }
        }
    }
}
