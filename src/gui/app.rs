use eframe::egui;
use parking_lot::Mutex;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;

use super::{file_viewer::FileViewer, process_selector::ProcessSelector, theme};
use crate::analysis::{self, AnalysisResult};
use crate::output::Output;

#[derive(Clone, Debug)]
pub enum LogLevel {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Clone, Debug)]
pub struct LogEntry {
    pub level: LogLevel,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

pub struct DumperApp {
    process_selector: ProcessSelector,
    file_viewer: FileViewer,
    logs: Arc<Mutex<Vec<LogEntry>>>,
    progress: Arc<Mutex<f32>>,
    is_dumping: Arc<Mutex<bool>>,
    dump_complete: bool,
    output_dir: PathBuf,
    animation_time: f32,
}

impl DumperApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::setup_custom_fonts(&cc.egui_ctx);
        theme::apply_dark_theme(&cc.egui_ctx);

        let output_dir = PathBuf::from("output");

        Self {
            process_selector: ProcessSelector::new(),
            file_viewer: FileViewer::new(output_dir.clone()),
            logs: Arc::new(Mutex::new(Vec::new())),
            progress: Arc::new(Mutex::new(0.0)),
            is_dumping: Arc::new(Mutex::new(false)),
            dump_complete: false,
            output_dir,
            animation_time: 0.0,
        }
    }

    fn add_log(&self, level: LogLevel, message: String) {
        let entry = LogEntry {
            level,
            message,
            timestamp: chrono::Utc::now(),
        };
        self.logs.lock().push(entry);
    }

    fn start_dump(&mut self, process_name: String) {
        if *self.is_dumping.lock() {
            return;
        }

        *self.is_dumping.lock() = true;
        *self.progress.lock() = 0.0;
        self.logs.lock().clear();
        self.dump_complete = false;

        let logs = self.logs.clone();
        let progress = self.progress.clone();
        let is_dumping = self.is_dumping.clone();
        let output_dir = self.output_dir.clone();

        thread::spawn(move || {
            logs.lock().push(LogEntry {
                level: LogLevel::Info,
                message: format!("Starting dump for process: {}", process_name),
                timestamp: chrono::Utc::now(),
            });

            *progress.lock() = 0.1;

            #[cfg(windows)]
            {
                use memflow::prelude::v1::*;

                let result = (|| -> anyhow::Result<()> {
                    let os = memflow_native::create_os(&OsArgs::default(), LibArc::default())?;
                    let mut process = os.into_process_by_name(&process_name)?;

                    logs.lock().push(LogEntry {
                        level: LogLevel::Success,
                        message: "Process attached successfully".to_string(),
                        timestamp: chrono::Utc::now(),
                    });

                    *progress.lock() = 0.2;

                    logs.lock().push(LogEntry {
                        level: LogLevel::Info,
                        message: "Analyzing buttons...".to_string(),
                        timestamp: chrono::Utc::now(),
                    });

                    let result = analysis::analyze_all(&mut process)?;

                    *progress.lock() = 0.8;

                    logs.lock().push(LogEntry {
                        level: LogLevel::Success,
                        message: format!("Found {} buttons", result.buttons.len()),
                        timestamp: chrono::Utc::now(),
                    });

                    logs.lock().push(LogEntry {
                        level: LogLevel::Success,
                        message: format!(
                            "Found {} interfaces across {} modules",
                            result
                                .interfaces
                                .iter()
                                .map(|(_, ifaces)| ifaces.len())
                                .sum::<usize>(),
                            result.interfaces.len()
                        ),
                        timestamp: chrono::Utc::now(),
                    });

                    logs.lock().push(LogEntry {
                        level: LogLevel::Success,
                        message: format!(
                            "Found {} offsets across {} modules",
                            result
                                .offsets
                                .iter()
                                .map(|(_, offsets)| offsets.len())
                                .sum::<usize>(),
                            result.offsets.len()
                        ),
                        timestamp: chrono::Utc::now(),
                    });

                    logs.lock().push(LogEntry {
                        level: LogLevel::Success,
                        message: format!(
                            "Found {} signatures across {} modules",
                            result
                                .signatures
                                .iter()
                                .map(|(_, sigs)| sigs.len())
                                .sum::<usize>(),
                            result.signatures.len()
                        ),
                        timestamp: chrono::Utc::now(),
                    });

                    let (class_count, enum_count) = result.schemas.values().fold(
                        (0, 0),
                        |(classes, enums), (class_vec, enum_vec)| {
                            (classes + class_vec.len(), enums + enum_vec.len())
                        },
                    );

                    logs.lock().push(LogEntry {
                        level: LogLevel::Success,
                        message: format!(
                            "Found {} classes and {} enums across {} modules",
                            class_count,
                            enum_count,
                            result.schemas.len()
                        ),
                        timestamp: chrono::Utc::now(),
                    });

                    *progress.lock() = 0.9;

                    logs.lock().push(LogEntry {
                        level: LogLevel::Info,
                        message: "Writing output files...".to_string(),
                        timestamp: chrono::Utc::now(),
                    });

                    let file_types = vec![
                        "cs".to_string(),
                        "hpp".to_string(),
                        "json".to_string(),
                        "rs".to_string(),
                        "zig".to_string(),
                    ];

                    let output = Output::new(&file_types, 4, &output_dir, &result)?;
                    output.dump_all(&mut process)?;

                    *progress.lock() = 1.0;

                    logs.lock().push(LogEntry {
                        level: LogLevel::Success,
                        message: "✓ Dump completed successfully!".to_string(),
                        timestamp: chrono::Utc::now(),
                    });

                    Ok(())
                })();

                if let Err(e) = result {
                    logs.lock().push(LogEntry {
                        level: LogLevel::Error,
                        message: format!("Dump failed: {}", e),
                        timestamp: chrono::Utc::now(),
                    });
                    *progress.lock() = 0.0;
                }
            }

            #[cfg(not(windows))]
            {
                logs.lock().push(LogEntry {
                    level: LogLevel::Error,
                    message: "Dumping is only supported on Windows".to_string(),
                    timestamp: chrono::Utc::now(),
                });
            }

            *is_dumping.lock() = false;
        });
    }
}

impl eframe::App for DumperApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.animation_time += ctx.input(|i| i.stable_dt);

        if *self.is_dumping.lock() {
            ctx.request_repaint();
        }

        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.add_space(20.0);
                ui.label(theme::heading("CS2 Dumper"));
                ui.add_space(10.0);

                let pulse = (self.animation_time * 2.0).sin() * 0.5 + 0.5;
                let color = egui::Color32::from_rgb(
                    (100.0 + pulse * 155.0) as u8,
                    (150.0 + pulse * 105.0) as u8,
                    250,
                );

                ui.colored_label(color, "⚡");
            });
            ui.add_space(10.0);
        });

        egui::SidePanel::left("process_panel")
            .resizable(false)
            .exact_width(400.0)
            .show(ctx, |ui| {
                ui.add_space(10.0);
                self.process_selector.ui(ui);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.add_space(10.0);

                    let can_dump = self.process_selector.get_selected().is_some()
                        && !*self.is_dumping.lock();

                    ui.add_enabled_ui(can_dump, |ui| {
                        let button = egui::Button::new(
                            egui::RichText::new("🚀 Start Dump")
                                .size(18.0)
                                .color(egui::Color32::WHITE),
                        )
                        .min_size(egui::vec2(150.0, 40.0));

                        if ui.add(button).clicked() {
                            if let Some(process_name) = self.process_selector.get_selected() {
                                self.start_dump(process_name);
                            }
                        }
                    });

                    ui.add_space(20.0);

                    if self.dump_complete || *self.progress.lock() >= 1.0 {
                        if ui
                            .button(
                                egui::RichText::new("📁 Open Output Folder")
                                    .size(16.0)
                                    .color(egui::Color32::WHITE),
                            )
                            .clicked()
                        {
                            #[cfg(windows)]
                            {
                                let _ = std::process::Command::new("explorer")
                                    .arg(&self.output_dir)
                                    .spawn();
                            }

                            #[cfg(not(windows))]
                            {
                                let _ = std::process::Command::new("xdg-open")
                                    .arg(&self.output_dir)
                                    .spawn();
                            }
                        }
                    }
                });

                ui.add_space(10.0);

                let progress_val = *self.progress.lock();
                if progress_val > 0.0 {
                    ui.horizontal(|ui| {
                        ui.add_space(10.0);
                        let progress_bar = egui::ProgressBar::new(progress_val)
                            .text(format!("{:.0}%", progress_val * 100.0))
                            .animate(*self.is_dumping.lock());
                        ui.add_sized(egui::vec2(300.0, 20.0), progress_bar);
                    });
                    ui.add_space(5.0);
                }

                ui.separator();

                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.set_width(ui.available_width() * 0.5 - 10.0);
                        ui.label(theme::subheading("Log Output"));
                        ui.separator();

                        egui::ScrollArea::vertical()
                            .auto_shrink([false; 2])
                            .stick_to_bottom(true)
                            .show(ui, |ui| {
                                for entry in self.logs.lock().iter() {
                                    let time_str =
                                        entry.timestamp.format("%H:%M:%S").to_string();

                                    let (icon, color) = match entry.level {
                                        LogLevel::Info => ("ℹ", egui::Color32::from_rgb(100, 150, 250)),
                                        LogLevel::Success => {
                                            ("✓", egui::Color32::from_rgb(100, 255, 100))
                                        }
                                        LogLevel::Warning => {
                                            ("⚠", egui::Color32::from_rgb(255, 200, 100))
                                        }
                                        LogLevel::Error => {
                                            ("✗", egui::Color32::from_rgb(255, 100, 100))
                                        }
                                    };

                                    ui.horizontal(|ui| {
                                        ui.colored_label(
                                            egui::Color32::from_rgb(150, 150, 150),
                                            format!("[{}]", time_str),
                                        );
                                        ui.colored_label(color, icon);
                                        ui.colored_label(color, &entry.message);
                                    });
                                }
                            });
                    });

                    ui.separator();

                    ui.vertical(|ui| {
                        if self.dump_complete || *self.progress.lock() >= 1.0 {
                            if !self.dump_complete {
                                self.file_viewer.refresh();
                                self.dump_complete = true;
                            }

                            self.file_viewer.ui(ui);
                        } else {
                            ui.centered_and_justified(|ui| {
                                ui.label(theme::body("Files will appear here after dump"));
                            });
                        }
                    });
                });
            });
        });
    }
}
