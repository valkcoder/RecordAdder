use arboard::Clipboard;
use eframe::egui;
use mlua::Lua;
use std::collections::HashMap;

#[derive(Clone)]
struct Record {
    player: String,
    time: f32,
    bounce: bool,
    obby: String,
}

#[derive(Default)]
struct AppState {
    player_input: String,
    time_input: String,
    obby_input: String,
    is_bounce: bool,
    ctt2_mode: bool,
    records: Vec<Record>,
    show_help: bool,
}

impl AppState {
    fn add_record_entry(&mut self, obby: &str, bounce: bool, player: &str, time: f32) {
        let new_record = Record {
            player: player.to_string(),
            time,
            bounce,
            obby: obby.to_string(),
        };

        if let Some(existing_index) = self
            .records
            .iter()
            .position(|r| r.obby == new_record.obby && r.bounce == new_record.bounce)
        {
            if self.records[existing_index].time > new_record.time {
                self.records[existing_index] = new_record;
            }
        } else {
            self.records.push(new_record);
        }
    }

    fn add_record(&mut self) {
        let obby = self.obby_input.clone();
        let player = self.player_input.clone();
        let bounce = self.is_bounce;

        if let Ok(time) = self.time_input.parse::<f32>() {
            self.add_record_entry(&obby, bounce, &player, time);

            self.player_input.clear();
            self.time_input.clear();
            self.obby_input.clear();
            self.is_bounce = false;
        }
    }

    fn import_from_clipboard(&mut self) {
        if let Ok(mut clipboard) = Clipboard::new() {
            if let Ok(content) = clipboard.get_text() {
                let lua = Lua::new();
                let result: mlua::Result<Vec<(String, bool, String, f32)>> = lua
                    .load(&format!("return {}", content))
                    .eval()
                    .and_then(|table: mlua::Table| {
                        let mut out = Vec::new();
                        for pair in table.pairs::<mlua::Value, mlua::Value>() {
                            if let Ok((key, value)) = pair {
                                if let mlua::Value::String(key_str) = &key {
                                    if key_str.to_str().ok() == Some("CTT2Mode") {
                                        continue;
                                    }
                                }

                                let obby_name = match &key {
                                    mlua::Value::String(s) => {
                                        s.to_str().unwrap_or_default().to_string()
                                    }
                                    _ => continue,
                                };

                                let mode_table = match value {
                                    mlua::Value::Table(t) => t,
                                    _ => continue,
                                };

                                for mode_pair in mode_table.pairs::<String, mlua::Table>() {
                                    let (mode, data) = mode_pair?;
                                    let bounce = mode == "Bounce";
                                    let player = data.get::<usize, String>(1)?;
                                    let time = data.get::<usize, f32>(2)?;
                                    out.push((obby_name.clone(), bounce, player, time));
                                }
                            }
                        }
                        Ok(out)
                    });

                if let Ok(entries) = result {
                    for (obby_name, bounce, player, time) in entries {
                        self.add_record_entry(&obby_name, bounce, &player, time);
                    }
                }
            }
        }
    }

    fn delete_record(&mut self, index: usize) {
        self.records.remove(index);
    }

    fn copy_to_clipboard(&self) {
        let mut map: HashMap<String, HashMap<String, (String, f32)>> = HashMap::new();

        for r in &self.records {
            let bounce_type = if r.bounce { "Bounce" } else { "Bounceless" };
            map.entry(r.obby.clone())
                .or_default()
                .insert(bounce_type.to_string(), (r.player.clone(), r.time));
        }

        let mut output = String::from("{\n");
        output.push_str(&format!(
            "  [\"CTT2Mode\"] = {},\n",
            if self.ctt2_mode { "true" } else { "false" }
        ));
        for (obby, types) in map {
            output.push_str(&format!("  [\"{}\"] = {{\n", obby));
            if let Some((player, time)) = types.get("Bounce") {
                output.push_str(&format!(
                    "    [\"Bounce\"] = {{ \"{}\", {:.3} }},\n",
                    player, time
                ));
            }
            if let Some((player, time)) = types.get("Bounceless") {
                output.push_str(&format!(
                    "    [\"Bounceless\"] = {{ \"{}\", {:.3} }},\n",
                    player, time
                ));
            }
            output.push_str("  },\n");
        }
        output.push_str("}");

        if let Ok(mut clipboard) = Clipboard::new() {
            clipboard.set_text(output).ok();
        }
    }
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let style = ui.style_mut();
                style.spacing.item_spacing = egui::vec2(12.0, 12.0);
                style.text_styles = [
                    (egui::TextStyle::Heading, egui::FontId::proportional(30.0)),
                    (egui::TextStyle::Body, egui::FontId::proportional(22.0)),
                    (egui::TextStyle::Button, egui::FontId::proportional(22.0)),
                    (egui::TextStyle::Small, egui::FontId::proportional(18.0)),
                ]
                .into();

                if self.show_help {
                    if ui.button("← Back").clicked() {
                        self.show_help = false;
                    }

                    ui.heading("If you can't see all the text, maximize the window or scroll.");

                    ui.separator();

                    ui.heading("How to Use (Rust App)");
                    ui.label("1. Enter player name, obby name, and time.");
                    ui.label("2. Toggle Bounce if it's a bounce record.");
                    ui.label("3. Click 'Add Record' to add it to the list.");
                    ui.label("4. Click 'Copy to Clipboard' to export in Lua format.");
                    ui.label("5. Use 'Import from Clipboard' to paste records from Roblox. (see roblox studio guide)");
                    ui.label("6. Use the Delete button to remove entries.");
                    ui.label("7. Use the 'CTT2 Mode' toggle if you're targeting the CTT2 folder structure in Roblox.");

                    ui.separator();

                    ui.heading("How to Use (Roblox Studio)");
                    ui.label("First of all, you need to download the record module. This app is basically useless without it.");
                    ui.label("1. Place the module in ReplicatedStorage.Modules.");
                    ui.label("2. Require it with: require(game.ReplicatedStorage.Modules.RecordModule)");
                    ui.label("3. To import: require(game.ReplicatedStorage.Modules.RecordModule).add(<table_from_clipboard>)");
                    ui.label("4. To export: Make a server script with the following content: print(game.ReplicatedStorage.Modules.RecordModule.get_records(true_or_false_for_ctt2mode))");
                    ui.label("5. If CTT2Mode is true, it will use workspace.MISC.LBS.[OBBYNAME:UPPER()].B or NB");
                    ui.label("6. If not, it uses the traditional '[ObbyName][Bounce|Bounceless]Leaderboard' format.");
                    return;
                }


                ui.horizontal(|ui| {
                    ui.label("Player Name:");
                    ui.text_edit_singleline(&mut self.player_input);
                });

                ui.horizontal(|ui| {
                    ui.label("Time (s):");
                    ui.text_edit_singleline(&mut self.time_input);
                });

                ui.horizontal(|ui| {
                    ui.label("Obby Name:");
                    ui.text_edit_singleline(&mut self.obby_input);
                });

                ui.checkbox(&mut self.is_bounce, "Bounce");

                if ui.button("Add Record").clicked() {
                    self.add_record();
                }

                ui.separator();
                
                ui.heading("World Records");

                let mut to_delete: Option<usize> = None;
                for (i, record) in self.records.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!(
                            "{} - {} - {} - {:.3}s",
                            record.obby,
                            if record.bounce {
                                "Bounce"
                            } else {
                                "Bounceless"
                            },
                            record.player,
                            record.time
                        ));
                        if ui.button("Delete").clicked() {
                            to_delete = Some(i);
                        }
                    });
                }
                if let Some(i) = to_delete {
                    self.delete_record(i);
                }

                ui.separator();

                if ui.button("Copy to Clipboard").clicked() {
                    self.copy_to_clipboard();
                }

                if ui.button("Import from Clipboard").clicked() {
                    self.import_from_clipboard();
                }

                ui.separator();

                ui.checkbox(&mut self.ctt2_mode, "CTT2 Mode");
                if ui.button("How to Use").clicked() {
                    self.show_help = true;
                }
            });
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Valk's Record Adder™",
        options,
        Box::new(|_cc| Box::<AppState>::default()),
    )
}
