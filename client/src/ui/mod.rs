use std::sync::mpsc::{Receiver, Sender};

use eframe::egui;

use crate::{
    comm::dto::EspDevice,
    ui::{config_window::ConfigWindow, setup::render_setup, showtime::ShowtimeApp},
};

mod config_window;
mod device_card;
mod setup;
mod showtime;

pub enum AppEvent {
    StartStreaming(Receiver<Vec<EspDevice>>),
    EditDevice(EspDevice),
    BackToMain,
}

pub enum AppState {
    Setup { host: String, port: String },
    SetConfig(ConfigWindow),
    Running(ShowtimeApp),
}

pub struct MainWrapper {
    state: AppState,
    event_tx: Sender<AppEvent>,
    event_rx: Receiver<AppEvent>,
    mqtt: Option<Receiver<Vec<EspDevice>>>,
}

impl MainWrapper {
    pub fn new() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        Self {
            state: AppState::Setup {
                host: "localhost".to_string(),
                port: "1883".to_string(),
            },
            event_rx: rx,
            event_tx: tx,
            mqtt: None,
        }
    }
}

impl eframe::App for MainWrapper {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        while let Ok(event) = self.event_rx.try_recv() {
            match event {
                AppEvent::StartStreaming(rx) => {
                    self.state = AppState::Running(ShowtimeApp::new(rx, self.event_tx.clone()));
                }
                AppEvent::EditDevice(dev) => {
                    // Extrahiere den Receiver aus dem Running-State
                    let config_window = ConfigWindow::new(dev.clone());
                    if let AppState::Running(app) = std::mem::replace(
                        &mut self.state,
                        AppState::SetConfig(config_window),
                    ) {
                        self.mqtt = Some(app.rx);
                    } else {
                        self.state = AppState::SetConfig(ConfigWindow::new(dev));
                    }
                }
                AppEvent::BackToMain => {
                    if let Some(rx) = self.mqtt.take() {
                        self.state = AppState::Running(ShowtimeApp::new(rx, self.event_tx.clone()));
                    } else {
                        self.state = AppState::Setup {
                            host: "localhost".to_string(),
                            port: "1883".to_string(),
                        };
                    }
                }
            }
        }

        match &mut self.state {
            AppState::Setup { host, port } => {
                let tx = self.event_tx.clone();
                render_setup(ctx, host, port, move |rx| {
                    let _ = tx.send(AppEvent::StartStreaming(rx));
                });
            }
            AppState::SetConfig(config_window) => {
                let tx = self.event_tx.clone();
                config_window.render(ctx, move || {
                    let _ = tx.send(AppEvent::BackToMain);
                });
            }
            AppState::Running(app) => {
                app.update(ctx, frame);
            }
        }

        ctx.request_repaint();
    }
}
