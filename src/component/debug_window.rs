
#[derive(serde::Deserialize, serde::Serialize)]
pub struct EguiWindows {
    // egui stuff:
    settings: bool,
    inspection: bool,
    memory: bool,
    output_events: bool,

    #[serde(skip)]
    output_event_history: std::collections::VecDeque<egui::output::OutputEvent>,
}

impl Default for EguiWindows {
    fn default() -> Self {
        EguiWindows::none()
    }
}

impl EguiWindows {
    fn none() -> Self {
        Self {
            settings: false,
            inspection: false,
            memory: false,
            output_events: false,
            output_event_history: Default::default(),
        }
    }

    pub fn checkboxes(&mut self, ui: &mut egui::Ui) {
        let Self {
            settings,
            inspection,
            memory,
            output_events,
            output_event_history: _,
        } = self;

        ui.checkbox(settings, "🔧 Settings");
        ui.checkbox(inspection, "🔍 Inspection");
        ui.checkbox(memory, "📝 Memory");
        ui.checkbox(output_events, "📤 Output Events");
    }

    pub fn windows(&mut self, ctx: &egui::Context) {
        let Self {
            settings,
            inspection,
            memory,
            output_events,
            output_event_history,
        } = self;

        ctx.output(|o| {
            for event in &o.events {
                output_event_history.push_back(event.clone());
            }
        });
        while output_event_history.len() > 1000 {
            output_event_history.pop_front();
        }

        egui::Window::new("🔧 Settings")
            .open(settings)
            .vscroll(true)
            .show(ctx, |ui| {
                ctx.settings_ui(ui);
            });

        egui::Window::new("🔍 Inspection")
            .open(inspection)
            .vscroll(true)
            .show(ctx, |ui| {
                ctx.inspection_ui(ui);
            });

        egui::Window::new("📝 Memory")
            .open(memory)
            .resizable(false)
            .show(ctx, |ui| {
                ctx.memory_ui(ui);
            });

        egui::Window::new("📤 Output Events")
            .open(output_events)
            .resizable(true)
            .default_width(520.0)
            .show(ctx, |ui| {
                ui.label(
                    "Recent output events from egui. \
            These are emitted when you interact with widgets, or move focus between them with TAB. \
            They can be hooked up to a screen reader on supported platforms.",
                );

                ui.separator();

                egui::ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        for event in output_event_history {
                            ui.label(format!("{event:?}"));
                        }
                    });
            });
    }
}