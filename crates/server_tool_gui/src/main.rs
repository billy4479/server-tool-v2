#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use anyhow::Result;
use eframe::egui;
use server_tool_lib::config::Config;

fn main() -> Result<()> {
    pretty_env_logger::init();

    let options = eframe::NativeOptions::default();
    let config = Config::load()?;

    eframe::run_native(
        "Server Tool V2",
        options,
        Box::new(|_cc| Box::new(MyApp::new(config))),
    );

    Ok(())
}

struct MyApp {
    pub config: Config,
}

impl MyApp {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Server Tool");
            if ui.button("Start a server").clicked() {}
        });
    }

    fn on_close_event(&mut self) -> bool {
        self.config
            .write()
            .unwrap_or_else(|_| log::error!("Error while writing the config"));
        true
    }
}
