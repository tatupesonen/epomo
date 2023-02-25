use std::{fmt::Display, time::Duration};

use egui::{Button, Color32};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct EpomoApp {
    // this how you opt-out of serialization of a member
    interval_period: i64,
    long_break_period: i64,
    short_break_period: i64,
    session_count: usize,

    #[serde(skip)]
    current_mode: PomodoroMode,
    #[serde(skip)]
    ends_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Default for EpomoApp {
    fn default() -> Self {
        Self {
            interval_period: 25,
            long_break_period: 15,
            short_break_period: 5,
            session_count: 0,
            ends_at: None,
            current_mode: PomodoroMode::Work, // Begin with work
        }
    }
}

impl EpomoApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

#[derive(Copy, Clone, PartialEq)]
enum PomodoroMode {
    LongBreak,
    ShortBreak,
    Work,
}

impl Display for PomodoroMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            PomodoroMode::LongBreak => write!(f, "Long break"),
            PomodoroMode::ShortBreak => write!(f, "Short break"),
            PomodoroMode::Work => write!(f, "Work"),
        }
    }
}

impl From<PomodoroMode> for egui::Color32 {
    fn from(val: PomodoroMode) -> Self {
        match val {
            PomodoroMode::LongBreak => Color32::from_rgb(240, 140, 58),
            PomodoroMode::ShortBreak => Color32::from_rgb(240, 231, 58),
            PomodoroMode::Work => Color32::from_rgb(58, 191, 240),
        }
    }
}

fn format_duration(duration: chrono::Duration, mode: PomodoroMode) -> String {
    format!(
        "{:02}:{:02}:{:02} {}",
        duration.num_hours(),
        duration.num_minutes(),
        duration.num_seconds() % 60,
        mode
    )
}

fn get_mode(cur_mode: PomodoroMode, session_count: usize) -> PomodoroMode {
    match cur_mode {
        PomodoroMode::Work => {
            if session_count % 4 == 0 {
                PomodoroMode::LongBreak
            } else {
                PomodoroMode::ShortBreak
            }
        }
        _ => PomodoroMode::Work,
    }
}

impl eframe::App for EpomoApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self {
            interval_period,
            ends_at,
            long_break_period,
            short_break_period,
            current_mode,
            session_count,
        } = self;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Pomodoro");
            ui.vertical(|ui| {
                ui.label("Interval time in minutes");
                ui.add_enabled(
                    ends_at.is_none() || *current_mode != PomodoroMode::Work,
                    egui::Slider::new(interval_period, 1..=120).suffix("m"),
                );
            });
            ui.vertical(|ui| {
                ui.label("Short break time in minutes");
                ui.add_enabled(
                    *current_mode != PomodoroMode::ShortBreak,
                    egui::Slider::new(short_break_period, 1..=30).suffix("m"),
                );
            });
            ui.vertical(|ui| {
                ui.label("Long break time in minutes");
                ui.add_enabled(
                    *current_mode != PomodoroMode::LongBreak,
                    egui::Slider::new(long_break_period, 1..=120).suffix("m"),
                );
            });
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(ends_at.is_none(), Button::new("Start"))
                    .clicked()
                {
                    *ends_at =
                        Some(chrono::Utc::now() + chrono::Duration::minutes(*interval_period));
                };
                if ui
                    .add_enabled(ends_at.is_some(), Button::new("Stop"))
                    .clicked()
                {
                    *ends_at = None;
                    *session_count = 0;
                };
            });
            // Countdown
            if ends_at.is_some() {
                // Core loop
                let now = chrono::Utc::now();
                let time_left = ends_at.unwrap() - now;
                if time_left < chrono::Duration::seconds(0) {
                    if *current_mode == PomodoroMode::Work {
                        *session_count += 1;
                    }
                    *current_mode = get_mode(*current_mode, *session_count);
                    match current_mode {
                        PomodoroMode::LongBreak => {
                            *ends_at = Some(
                                chrono::Utc::now() + chrono::Duration::minutes(*long_break_period),
                            )
                        }
                        PomodoroMode::ShortBreak => {
                            *ends_at = Some(
                                chrono::Utc::now() + chrono::Duration::minutes(*short_break_period),
                            )
                        }
                        PomodoroMode::Work => {
                            *ends_at = Some(
                                chrono::Utc::now() + chrono::Duration::minutes(*interval_period),
                            )
                        }
                    }
                    ctx.request_repaint();
                }
                ui.label(
                    egui::RichText::new(format_duration(time_left, *current_mode))
                        .heading()
                        .color(Into::<Color32>::into(*current_mode)),
                );
                ui.label(format!("Completed session count {}", *session_count));
            }
            ctx.request_repaint_after(Duration::from_secs(1));
        });
    }
}
