use eframe::egui;
use parking_lot::Mutex;
use std::sync::Arc;
use std::thread;

// НАШ локальный тип
#[derive(Clone, Debug)]
pub struct MyProcessInfo {
    pub name: String,
    pub pid: u32,
}

pub struct ProcessSelector {
    processes: Arc<Mutex<Vec<MyProcessInfo>>>,
    selected_process: Option<String>,
    search_query: String,
    loading: Arc<Mutex<bool>>,
}

impl ProcessSelector {
    pub fn new() -> Self {
        Self {
            processes: Arc::new(Mutex::new(Vec::new())),
            selected_process: None,
            search_query: String::new(),
            loading: Arc::new(Mutex::new(false)),
        }
    }

    pub fn refresh_processes(&mut self) {
        let mut loading = self.loading.lock();
        if *loading {
            return;
        }
        *loading = true;
        drop(loading);

        let processes = self.processes.clone();
        let loading_flag = self.loading.clone();

        thread::spawn(move || {
            let mut proc_list: Vec<MyProcessInfo> = Vec::new();

            #[cfg(windows)]
            {
                use memflow::prelude::v1::*;

                if let Ok(os) = memflow_native::create_os(&OsArgs::default(), LibArc::default()) {
                    if let Ok(list) = os.process_info_list() {
                        for info in list {
                            // Конвертируем правильно
                            let name_str = info.name.to_string();
                            let pid_val: u32 = info.pid.into();
                            
                            proc_list.push(MyProcessInfo {
                                name: name_str,
                                pid: pid_val,
                            });
                        }
                    }
                }
            }

            #[cfg(not(windows))]
            {
                use std::process::Command;

                if let Ok(output) = Command::new("ps").args(&["aux"]).output() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    for line in stdout.lines().skip(1) {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() > 10 {
                            if let Ok(pid) = parts[1].parse::<u32>() {
                                let name = parts[10..].join(" ");
                                proc_list.push(MyProcessInfo { name, pid });
                            }
                        }
                    }
                }
            }

            proc_list.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
            *processes.lock() = proc_list;
            *loading_flag.lock() = false;
        });
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) -> Option<String> {
        ui.vertical(|ui| {
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.label("Process Selection");
                ui.add_space(10.0);

                let is_loading = *self.loading.lock();
                if is_loading {
                    ui.label("Loading processes...");
                } else if ui.button("Refresh").clicked() {
                    self.refresh_processes();
                }
            });

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.label("Search:");
                ui.text_edit_singleline(&mut self.search_query);
            });

            ui.add_space(10.0);

            let is_loading = *self.loading.lock();
            let processes = self.processes.lock();
            
            if !is_loading && !processes.is_empty() {
                let cs2_exists = processes.iter().any(|p| p.name.to_lowercase().contains("cs2"));

                if cs2_exists && self.selected_process.is_none() {
                    if let Some(cs2_proc) = processes.iter().find(|p| p.name.to_lowercase().contains("cs2")) {
                        self.selected_process = Some(cs2_proc.name.clone());
                    }
                }

                if !cs2_exists {
                    ui.label("⚠ cs2.exe not found - please select process manually");
                }

                egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                    for proc in processes.iter() {
                        if !self.search_query.is_empty()
                            && !proc.name.to_lowercase().contains(&self.search_query.to_lowercase())
                        {
                            continue;
                        }

                        let is_selected = self.selected_process.as_ref().map(|s| s == &proc.name).unwrap_or(false);

                        if ui.selectable_label(is_selected, format!("{} (PID: {})", proc.name, proc.pid)).clicked() {
                            self.selected_process = Some(proc.name.clone());
                        }
                    }
                });
            } else if is_loading {
                ui.label("Loading process list...");
            }

            ui.add_space(10.0);

            if let Some(selected) = &self.selected_process {
                ui.label(format!("✓ Selected: {}", selected));
            }
        });

        self.selected_process.clone()
    }

    pub fn get_selected(&self) -> Option<String> {
        self.selected_process.clone()
    }
}
