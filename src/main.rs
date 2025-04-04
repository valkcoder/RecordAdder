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

struct AppState {
    player_input: String,
    time_input: String,
    obby_input: String,
    is_bounce: bool,
    ctt2_mode: bool,
    records: Vec<Record>,
    show_help: bool,

    main_player_input: String,
    main_time_input: String,
    main_category: String,

    main_ob_bounce: Vec<(String, f32)>,
    main_ob_bounceless: Vec<(String, f32)>,
    main_ob_noplat: Vec<(String, f32)>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            player_input: String::new(),
            time_input: String::new(),
            obby_input: String::new(),
            is_bounce: false,
            ctt2_mode: false,
            records: Vec::new(),
            show_help: false,

            main_player_input: String::new(),
            main_time_input: String::new(),
            main_category: "Bounce".to_string(),

            main_ob_bounce: Vec::new(),
            main_ob_bounceless: Vec::new(),
            main_ob_noplat: Vec::new(),
        }
    }
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

    fn add_main_ob_record(&mut self, player: String, time: f32, category: &str) {
        let list = match category {
            "Bounce" => &mut self.main_ob_bounce,
            "Bounceless" => &mut self.main_ob_bounceless,
            "NoPlat" => &mut self.main_ob_noplat,
            _ => return,
        };

        list.push((player, time));
        list.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        let max_len = match category {
            "Bounce" => 12,
            "Bounceless" => 11,
            "NoPlat" => 10,
            _ => 0,
        };

        if list.len() > max_len {
            list.truncate(max_len);
        }
    }

    fn import_from_clipboard(&mut self) {
        if let Ok(mut clipboard) = Clipboard::new() {
            if let Ok(content) = clipboard.get_text() {
                let lua = Lua::new();
                let result: mlua::Result<mlua::Table> =
                    lua.load(&format!("return {}", content)).eval();

                if let Ok(table) = result {
                    for pair in table.pairs::<mlua::Value, mlua::Value>() {
                        if let Ok((key, value)) = pair {
                            if let mlua::Value::String(key_str) = &key {
                                if key_str.to_str().ok() == Some("CTT2Mode") {
                                    continue;
                                }
                            }

                            if let mlua::Value::String(name) = &key {
                                let name_str = name.to_str().unwrap_or_default();
                                if name_str == "MainObby" {
                                    if let mlua::Value::Table(main_ob) = value {
                                        for cat in ["Bounce", "Bounceless", "NoPlat"] {
                                            if let Ok(sub) = main_ob.get::<_, mlua::Table>(cat) {
                                                for r in sub.sequence_values::<mlua::Table>() {
                                                    if let Ok(entry) = r {
                                                        let player = entry
                                                            .get::<_, String>(1)
                                                            .unwrap_or_default();
                                                        let time = entry
                                                            .get::<_, f32>(2)
                                                            .unwrap_or(9999.0);
                                                        self.add_main_ob_record(player, time, cat);
                                                    }
                                                }
                                            }
                                        }
                                    }
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
                                let (mode, data) = mode_pair.unwrap();
                                let bounce = mode == "Bounce";
                                let player = data.get::<usize, String>(1).unwrap_or_default();
                                let time = data.get::<usize, f32>(2).unwrap_or(9999.0);
                                self.add_record_entry(&obby_name, bounce, &player, time);
                            }
                        }
                    }
                }
            }
        }
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

        for (obby, types) in &map {
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

        if self.ctt2_mode {
            output.push_str("  [\"MainObby\"] = {\n");

            let write_cat = |name: &str, list: &Vec<(String, f32)>, out: &mut String| {
                if !list.is_empty() {
                    out.push_str(&format!("    [\"{}\"] = {{\n", name));
                    for (p, t) in list {
                        out.push_str(&format!("      {{ \"{}\", {:.3} }},\n", p, t));
                    }
                    out.push_str("    },\n");
                }
            };

            write_cat("Bounce", &self.main_ob_bounce, &mut output);
            write_cat("Bounceless", &self.main_ob_bounceless, &mut output);
            write_cat("NoPlat", &self.main_ob_noplat, &mut output);

            output.push_str("  },\n");
        }

        output.push_str("}");

        if let Ok(mut clipboard) = Clipboard::new() {
            clipboard.set_text(output).ok();
        }
    }

    fn delete_record(&mut self, index: usize) {
        self.records.remove(index);
    }
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("World Record Editor");

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
                
                    ui.heading("How to Use (Main Obby Records)");
                    ui.label("Only visible when CTT2 Mode is enabled.");
                    ui.label("1. Choose player, time and category (Bounce, Bounceless or NoPlat).");
                    ui.label("2. Max 12 for Bounce, 11 for Bounceless, 10 for NoPlat.");
                    ui.label("3. Records are sorted automatically by time.");
                    ui.label("4. Export will include them in the MainObby section.");
                
                    ui.separator();
                
                    ui.heading("How to Use (Roblox Studio)");
                    ui.label("1. Download the RecordModule and place it in ReplicatedStorage.Modules.");
                    ui.label("2. Require it with: require(game.ReplicatedStorage.Modules.RecordModule)");
                    ui.label("3. To import: require(game.ReplicatedStorage.Modules.RecordModule).add(<table_from_clipboard>)");
                    ui.label("4. To export: print(require(game.ReplicatedStorage.Modules.RecordModule).get_records(true_or_false_for_ctt2mode))");
                    ui.label("5. If CTT2Mode is true, it will use workspace.MISC.LBS.[OBBYNAME:UPPER()].B or NB");
                    ui.label("6. If not, it uses the traditional '[ObbyName][Bounce|Bounceless]Leaderboard' format.");
                    ui.label("7. For MainObby, it updates MISC.LBS.MO.[B/NB/NT].LB.Leaderboard.ScrollingFrame entries 1–12.");
                
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
                ui.heading("Records");

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

                if self.ctt2_mode {
                    ui.separator();
                    ui.heading("Main Obby Records");

                    ui.horizontal(|ui| {
                        ui.label("Player Name:");
                        ui.text_edit_singleline(&mut self.main_player_input);
                    });

                    ui.horizontal(|ui| {
                        ui.label("Time (s):");
                        ui.text_edit_singleline(&mut self.main_time_input);
                    });

                    ui.horizontal(|ui| {
                        ui.label("Category:");
                        egui::ComboBox::from_id_source("main_category")
                            .selected_text(&self.main_category)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut self.main_category,
                                    "Bounce".to_string(),
                                    "Bounce",
                                );
                                ui.selectable_value(
                                    &mut self.main_category,
                                    "Bounceless".to_string(),
                                    "Bounceless",
                                );
                                ui.selectable_value(
                                    &mut self.main_category,
                                    "NoPlat".to_string(),
                                    "NoPlat",
                                );
                            });
                    });

                    if ui.button("Add Main Obby Record").clicked() {
                        if let Ok(t) = self.main_time_input.parse::<f32>() {
                            let cat = self.main_category.clone();
                            let player = self.main_player_input.clone();
                            self.add_main_ob_record(player, t, &cat);
                            self.main_player_input.clear();
                            self.main_time_input.clear();
                        }
                    }

                    for (label, list) in [
                        ("Bounce", &self.main_ob_bounce),
                        ("Bounceless", &self.main_ob_bounceless),
                        ("NoPlat", &self.main_ob_noplat),
                    ] {
                        ui.group(|ui| {
                            ui.heading(label);
                            for (i, (p, t)) in list.iter().enumerate() {
                                ui.label(format!("{}. {} - {:.3}s", i + 1, p, t));
                            }
                        });
                    }
                }

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
