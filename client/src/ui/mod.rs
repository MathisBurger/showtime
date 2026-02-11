use eframe::egui;

use crate::ui::{setup::render_setup, showtime::ShowtimeApp};

mod device_card;
mod setup;
mod showtime;

pub enum AppState {
    Setup { host: String, port: String },
    Running(ShowtimeApp),
}

pub struct MainWrapper {
    state: AppState,
}

impl MainWrapper {
    pub fn new() -> Self {
        Self {
            state: AppState::Setup {
                host: "localhost".to_string(),
                port: "1883".to_string(),
            },
        }
    }
}

impl eframe::App for MainWrapper {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let mut connection_rx = None;

        match &mut self.state {
            AppState::Setup { host, port } => {
                render_setup(ctx, host, port, |rx| {
                    connection_rx = Some(rx);
                });
            }
            AppState::Running(app) => {
                app.update(ctx, frame);
            }
        }
        if let Some(rx) = connection_rx {
            self.state = AppState::Running(ShowtimeApp::new(rx));
        }
    }
}
