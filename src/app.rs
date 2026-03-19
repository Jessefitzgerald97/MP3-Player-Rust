use std::default;

use egui::{pos2, Button, Rect, RichText};
use egui_extras::install_image_loaders;
use crate::audio::AudioManager;


/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    label: String,

    #[serde(skip)] // This how you opt-out of serialization of a field
    value: f32,
    #[serde(skip)] 
    is_playing: bool,
    #[serde(skip)]
    audio_manager: Option<AudioManager>,
    music_directory: Option<String>,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "bruh".to_owned(),
            value: 6.9,
            is_playing: false,
            audio_manager: None,
            music_directory: None,
        }
    }
}

impl TemplateApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals {
            panel_fill: egui::Color32::from_rgb(99, 16, 62),
            override_text_color: Some(egui::Color32::WHITE),
            ..Default::default()
        });

        install_image_loaders(&cc.egui_ctx);

        let mut app: TemplateApp = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        };

        // Initialize the audio manager
        app.audio_manager = AudioManager::new().ok();

        // If we don't have a music directory saved, prompt for one
        if app.music_directory.is_none() {
            if let Some(path) = rfd::FileDialog::new()
                .set_title("Select Music Directory")
                    .pick_folder() 
            {
                app.music_directory = Some(path.display().to_string());
            }
        }

        // Now that we have the directory, scan for music and start playback
        if let Some(dir) = &app.music_directory {
            if let Some(audio_manager) = &mut app.audio_manager {
                // Scan the directory for music files
                if let Err(e) = audio_manager.scan_directory(dir) {
                    println!("Error scanning directory: {}", e);
                } else {
                    // If scanning succeeded, try to start playing the first song
                    if let Err(e) = audio_manager.play_song_at_index(0) {
                        println!("Error starting playback: {}", e);
                    } else {
                        // Update the playing state if successful
                        audio_manager.pause();
                    }
                }
            }
        }
        app
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Select Music Directory").clicked() {
                            // When clicked, open a directory picker dialog
                            if let Some(path) = rfd::FileDialog::new()
                                .set_title("Select Music Directory")
                                    .pick_folder() 
                            {
                                // Store the selected directory path
                                self.music_directory = Some(path.display().to_string());

                                // Rescan the music directory with the audio manager
                                if let Some(audio_manager) = &mut self.audio_manager {
                                    if let Err(e) = audio_manager.scan_directory(
                                        &self.music_directory.as_ref().unwrap()
                                    ) {
                                        println!("Error scanning directory: {}", e);
                                    }
                                }
                            }
                        }
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        if ui.button("YOOOOOO").clicked(){
                            println!("YOOOOOO");
                        };
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        let side_panel_width: f32 = 250.0;

        let mut side_panel = egui::SidePanel::left("Playlist").max_width(side_panel_width).min_width(side_panel_width);

        side_panel.show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let playlist_info = if let Some(audio_manager) = &mut self.audio_manager {
                    let mut index = 0;
                    let mut switch_song = false;
                    if let Some(playlist) = &audio_manager.current_playlist {
                        let songs = playlist.get_songs();
                        for (i, song) in songs.iter().enumerate() {
                            let title = song.song_name.clone();
                            let image = song.cover_img_path.clone();
                            // println!("{}", image);
                            if image.eq_ignore_ascii_case("../assets/default_cover.png"){
                                if ui.add(egui::Button::image_and_text(egui::Image::new(egui::include_image!("../assets/default_cover.png")).fit_to_exact_size(egui::vec2(50.0, 50.0)), title)
                                    .min_size(egui::vec2(250.0, 50.0))
                                    .fill(egui::Color32::from_rgb(99, 16, 62))
                                ).clicked() {
                                    index = i;
                                    switch_song = true;
                                    self.is_playing = true;
                                };
                            } else {
                                if ui.add(egui::Button::image_and_text(egui::Image::new(format!("file://{image}")).fit_to_exact_size(egui::vec2(50.0, 50.0)), title)
                                    .min_size(egui::vec2(250.0, 50.0))
                                    .fill(egui::Color32::from_rgb(99, 16, 62))
                                ).clicked() {
                                    index = i;
                                    switch_song = true;
                                    self.is_playing = true;
                                };
                            }
                            ui.add(egui::Separator::default());
                        }
                    }
                    if (switch_song){
                        audio_manager.play_song_at_index(index);
                    }
                };
            });
        });

        egui::CentralPanel::default()
            .show(ctx, |ui| {
                // Paint on background instead of laying out like a widget.

                egui::Image::new(egui::include_image!("../assets/app-bg.jpeg")).paint_at(
                    ui,
                    [
                    [0.0,             0.0            ].into(),
                    [1000., 800.].into()
                    ].into()
                );
                ui.heading("Audio Player");

                // Display current song information
                let current_song_info = if let Some(audio_manager) = &self.audio_manager {
                    if let Some(playlist) = &audio_manager.current_playlist {
                        if let Some(current_song) = playlist.get_current_song() {
                            format!("{} ({})", current_song.song_name, current_song.song_length)
                        } else {
                            "No song selected".to_string()
                        }
                    } else {
                        "No playlist loaded".to_string()
                    }
                } else {
                    "Audio player not initialized".to_string()
                };

                //get current some img path
                let current_song_img = if let Some(audio_manager) = &self.audio_manager {
                    if let Some(playlist) = &audio_manager.current_playlist {
                        if let Some(current_song) = playlist.get_current_song() {
                            format!("{}", current_song.cover_img_path)
                        } else {
                            "No song selected".to_string()
                        }
                    } else {
                        "No playlist loaded".to_string()
                    }
                } else {
                    "Audio player not initialized".to_string()
                };

                if (current_song_img.eq_ignore_ascii_case("../assets/default_cover.png")){
                    egui::Image::new(egui::include_image!("../assets/default_cover.png"))
                        .rounding(5.0)
                        .bg_fill(egui::Color32::WHITE)
                        .paint_at(ui, Rect{min: pos2(380., 100.), max: pos2(880., 600.)});
                }
                else{
                    egui::Image::new(format!("file://{current_song_img}"))
                        .rounding(5.0)
                        .paint_at(ui, Rect{min: pos2(380., 100.), max: pos2(880., 600.)});
                }

                ui.add_space(550.0);

                let play_button = egui::ImageButton::new(egui::Image::new(egui::include_image!("../assets/play.png"))
                    .rounding(150.)
                    .bg_fill(egui::Color32::WHITE));

                let pause_button = egui::ImageButton::new(egui::Image::new(egui::include_image!("../assets/pause.png"))
                    .rounding(150.)
                    .bg_fill(egui::Color32::WHITE));

                let prev_button = egui::ImageButton::new(egui::Image::new(egui::include_image!("../assets/back.png"))
                    .rounding(150.)
                    .bg_fill(egui::Color32::WHITE));

                let next_button = egui::ImageButton::new(egui::Image::new(egui::include_image!("../assets/next.png"))
                    .rounding(150.)
                    .bg_fill(egui::Color32::WHITE));

                let now_playing_text_horiozontal_spacing = 120.;
                //code for buttons
                ui.horizontal(|ui| {
                    ui.add_space(now_playing_text_horiozontal_spacing);
                    ui.label(RichText::new(current_song_info).size(30.));
                });

                ui.horizontal(|ui|{
                    ui.add_space(now_playing_text_horiozontal_spacing);
                    // sorry i didnt do this cus i cant figure out how to get the artist
                    ui.label(RichText::new("artist").size(20.));
                });

                // Playback control buttons
                ui.horizontal_top(|ui| {
                    ui.add_space(120.0);

                    // Previous button
                    if ui.add_sized([165., 100.], prev_button).clicked() {
                        if let Some(audio_manager) = &mut self.audio_manager {
                            if let Err(e) = audio_manager.previous_song() {
                                println!("Error playing previous song: {}", e);
                            }
                            else{
                                self.is_playing = true;
                            }
                        }
                    }

                    // Play/Pause button
                    if ui.add_sized([165., 100.],
                        if self.is_playing { pause_button } else { play_button }
                    ).clicked() {
                        if let Some(audio_manager) = &mut self.audio_manager {
                            self.is_playing = !self.is_playing;
                            if self.is_playing {
                                if audio_manager.current_song.is_none() {
                                    if let Err(e) = audio_manager.play_current_song() {
                                        println!("Error starting playback: {}", e);
                                    }
                                } else {
                                    audio_manager.play();
                                    
                                }
                            } else {
                                audio_manager.pause();
                            }
                        }
                    }

                    // Next button
                    if ui.add_sized([165., 100.], next_button).clicked() {
                        if let Some(audio_manager) = &mut self.audio_manager {
                            if let Err(e) = audio_manager.next_song() {
                                println!("Error playing next song: {}", e);
                            }
                            else{
                                self.is_playing = true;
                            }
                        }
                    }
                });
            });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
