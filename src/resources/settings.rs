use bevy::prelude::*;
use bevy::window::PrimaryWindow;

const SETTINGS_FILE: &str = "settings.ini";

/// Настройки приложения: громкость и размер окна.
/// Загружаются из settings.ini при старте, сохраняются при изменении.
#[derive(Resource)]
pub struct AppSettings {
    pub music_volume:  f32, // 0.0 – 1.0
    pub sfx_volume:    f32, // 0.0 – 1.0
    pub window_width:  u32,
    pub window_height: u32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            music_volume:  0.7,
            sfx_volume:    0.8,
            window_width:  800,
            window_height: 600,
        }
    }
}

impl AppSettings {
    /// Загружает настройки из settings.ini.
    /// При отсутствии файла или ошибке возвращает Default.
    pub fn load() -> Self {
        let mut s = Self::default();
        let Ok(text) = std::fs::read_to_string(SETTINGS_FILE) else { return s };

        let mut section = String::new();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
                continue;
            }
            if line.starts_with('[') && line.ends_with(']') {
                section = line[1..line.len() - 1].to_ascii_lowercase();
                continue;
            }
            let Some((key, val)) = line.split_once('=') else { continue };
            let key = key.trim();
            let val = val.trim();
            match (section.as_str(), key) {
                ("audio", "music_volume") => {
                    if let Ok(v) = val.parse::<f32>() {
                        s.music_volume = v.clamp(0.0, 1.0);
                    }
                }
                ("audio", "sfx_volume") => {
                    if let Ok(v) = val.parse::<f32>() {
                        s.sfx_volume = v.clamp(0.0, 1.0);
                    }
                }
                ("window", "width") => {
                    if let Ok(v) = val.parse::<u32>() {
                        s.window_width = v.max(400);
                    }
                }
                ("window", "height") => {
                    if let Ok(v) = val.parse::<u32>() {
                        s.window_height = v.max(300);
                    }
                }
                _ => {}
            }
        }
        s
    }

    /// Сохраняет настройки в settings.ini.
    pub fn save(&self) {
        let content = format!(
            "[audio]\nmusic_volume={:.2}\nsfx_volume={:.2}\n\n[window]\nwidth={}\nheight={}\n",
            self.music_volume, self.sfx_volume, self.window_width, self.window_height,
        );
        let _ = std::fs::write(SETTINGS_FILE, content);
    }
}

/// Отслеживает изменение размера окна и обновляет AppSettings.
pub fn sync_window_size_system(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut settings: ResMut<AppSettings>,
) {
    let Ok(window) = windows.single() else { return };
    let w = window.width()  as u32;
    let h = window.height() as u32;
    if w != settings.window_width || h != settings.window_height {
        settings.window_width  = w;
        settings.window_height = h;
    }
}

/// Сохраняет AppSettings в файл, когда ресурс изменился.
pub fn save_settings_on_change(settings: Res<AppSettings>) {
    if settings.is_changed() && !settings.is_added() {
        settings.save();
    }
}
