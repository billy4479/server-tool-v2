#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use anyhow::Result;
use eframe::egui::{self, Label};
use poll_promise::Promise;
use server_tool::{
    config::Config,
    manifest::{self, VersionManifest},
};

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Server Tool V2",
        options,
        Box::new(|_cc| Box::<MyApp>::default()),
    );

    Ok(())
}

#[derive(Default)]
struct MyApp {
    config: Option<Promise<Result<Config>>>,
    manifest: Option<Promise<Result<Vec<VersionManifest>>>>,
}

impl MyApp {
    fn load_config(&mut self, ui: &mut egui::Ui) -> Option<&Config> {
        let config = self.config.get_or_insert_with(|| {
            let (sender, promise) = Promise::new();
            sender.send(Config::load());

            promise
        });

        match config.ready() {
            None => {
                ui.spinner();
                None
            }
            Some(Err(err)) => {
                ui.colored_label(ui.visuals().error_fg_color, err.to_string());
                None
            }
            Some(Ok(config)) => Some(config),
        }
    }

    fn load_manifest(&mut self, ui: &mut egui::Ui) -> Option<&Vec<VersionManifest>> {
        let manifest = self.manifest.get_or_insert_with(|| {
            let (sender, promise) = Promise::new();
            sender.send(futures::executor::block_on(manifest::get_version_infos()));

            promise
        });

        match manifest.ready() {
            None => {
                ui.spinner();
                None
            }
            Some(Err(err)) => {
                ui.colored_label(ui.visuals().error_fg_color, err.to_string());
                None
            }
            Some(Ok(manifest)) => Some(manifest),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let Some(config) = self.load_config(ui) else {
                return;
            };

            ui.label(config.to_yaml().expect("lol"));

            let Some(manifest) = self.load_manifest(ui) else {
                return;
            };

            egui::ScrollArea::both().show(ui, |ui| {
                ui.add(Label::new(format!("{:#?}", manifest)).wrap(false));
            });

            // ui.heading("Server Tool");
            // if ui.button("Reload manifest").clicked() {
            //     self.manifest =
            //         futures::executor::block_on(manifest::get_version_infos()).expect("oops");
            // }
            // if ui.button("Refetch manifest").clicked() {
            //     self.manifest =
            //         futures::executor::block_on(manifest::update_manifest()).expect("oops");
            // }
            // if ui.button("Reload config").clicked() {
            //     self.config = Config::load().expect("lol");
            // }
        });
    }

    fn on_close_event(&mut self) -> bool {
        if let Some(config) = &self.config {
            if let Some(Ok(config)) = config.ready() {
                config
                    .write()
                    .unwrap_or_else(|_| log::error!("Error while writing the config"));
            }
        }
        true
    }
}
