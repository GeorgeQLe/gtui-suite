use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorFormat {
    Hex,
    Rgb,
    Hsl,
    Hsv,
    Cmyk,
}

impl ColorFormat {
    pub fn name(&self) -> &'static str {
        match self {
            ColorFormat::Hex => "HEX",
            ColorFormat::Rgb => "RGB",
            ColorFormat::Hsl => "HSL",
            ColorFormat::Hsv => "HSV",
            ColorFormat::Cmyk => "CMYK",
        }
    }

    pub fn all() -> Vec<ColorFormat> {
        vec![
            ColorFormat::Hex,
            ColorFormat::Rgb,
            ColorFormat::Hsl,
            ColorFormat::Hsv,
            ColorFormat::Cmyk,
        ]
    }
}

#[derive(Debug, Clone)]
pub struct ColorValues {
    pub hex: String,
    pub rgb: (u8, u8, u8),
    pub hsl: (f64, f64, f64),
    pub hsv: (f64, f64, f64),
    pub cmyk: (f64, f64, f64, f64),
}

impl ColorValues {
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        let hex = format!("#{:02X}{:02X}{:02X}", r, g, b);
        let hsl = rgb_to_hsl(r, g, b);
        let hsv = rgb_to_hsv(r, g, b);
        let cmyk = rgb_to_cmyk(r, g, b);

        Self {
            hex,
            rgb: (r, g, b),
            hsl,
            hsv,
            cmyk,
        }
    }

    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return None;
        }

        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

        Some(Self::from_rgb(r, g, b))
    }
}

fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    let r = r as f64 / 255.0;
    let g = g as f64 / 255.0;
    let b = b as f64 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;

    if max == min {
        return (0.0, 0.0, l * 100.0);
    }

    let d = max - min;
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };

    let h = if max == r {
        ((g - b) / d + if g < b { 6.0 } else { 0.0 }) / 6.0
    } else if max == g {
        ((b - r) / d + 2.0) / 6.0
    } else {
        ((r - g) / d + 4.0) / 6.0
    };

    (h * 360.0, s * 100.0, l * 100.0)
}

fn rgb_to_hsv(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    let r = r as f64 / 255.0;
    let g = g as f64 / 255.0;
    let b = b as f64 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let d = max - min;

    let v = max;
    let s = if max == 0.0 { 0.0 } else { d / max };

    if max == min {
        return (0.0, s * 100.0, v * 100.0);
    }

    let h = if max == r {
        ((g - b) / d + if g < b { 6.0 } else { 0.0 }) / 6.0
    } else if max == g {
        ((b - r) / d + 2.0) / 6.0
    } else {
        ((r - g) / d + 4.0) / 6.0
    };

    (h * 360.0, s * 100.0, v * 100.0)
}

fn rgb_to_cmyk(r: u8, g: u8, b: u8) -> (f64, f64, f64, f64) {
    let r = r as f64 / 255.0;
    let g = g as f64 / 255.0;
    let b = b as f64 / 255.0;

    let k = 1.0 - r.max(g).max(b);

    if k == 1.0 {
        return (0.0, 0.0, 0.0, 100.0);
    }

    let c = (1.0 - r - k) / (1.0 - k);
    let m = (1.0 - g - k) / (1.0 - k);
    let y = (1.0 - b - k) / (1.0 - k);

    (c * 100.0, m * 100.0, y * 100.0, k * 100.0)
}

#[derive(Debug, Clone)]
pub struct SavedColor {
    pub name: String,
    pub values: ColorValues,
}

pub struct App {
    pub input: String,
    pub format: ColorFormat,
    pub current: Option<ColorValues>,
    pub saved: Vec<SavedColor>,
    pub selected_saved: usize,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            input: "FF5733".to_string(),
            format: ColorFormat::Hex,
            current: None,
            saved: vec![
                SavedColor {
                    name: "Red".to_string(),
                    values: ColorValues::from_rgb(255, 0, 0),
                },
                SavedColor {
                    name: "Green".to_string(),
                    values: ColorValues::from_rgb(0, 255, 0),
                },
                SavedColor {
                    name: "Blue".to_string(),
                    values: ColorValues::from_rgb(0, 0, 255),
                },
            ],
            selected_saved: 0,
            status_message: None,
        };
        app.parse_input();
        app
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return true;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Backspace => {
                self.input.pop();
                self.parse_input();
            }
            KeyCode::Char(c) if c.is_ascii_hexdigit() || c == ',' || c == ' ' || c == '.' => {
                self.input.push(c);
                self.parse_input();
            }
            KeyCode::Tab => {
                self.format = match self.format {
                    ColorFormat::Hex => ColorFormat::Rgb,
                    ColorFormat::Rgb => ColorFormat::Hsl,
                    ColorFormat::Hsl => ColorFormat::Hsv,
                    ColorFormat::Hsv => ColorFormat::Cmyk,
                    ColorFormat::Cmyk => ColorFormat::Hex,
                };
                self.input.clear();
            }
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.save_current();
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected_saved < self.saved.len().saturating_sub(1) {
                    self.selected_saved += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_saved = self.selected_saved.saturating_sub(1);
            }
            KeyCode::Enter => {
                if let Some(saved) = self.saved.get(self.selected_saved) {
                    self.current = Some(saved.values.clone());
                    self.input = saved.values.hex.trim_start_matches('#').to_string();
                    self.format = ColorFormat::Hex;
                }
            }
            KeyCode::Char('d') => {
                if !self.saved.is_empty() {
                    self.saved.remove(self.selected_saved);
                    self.selected_saved = self.selected_saved.min(self.saved.len().saturating_sub(1));
                    self.status_message = Some("Color removed".to_string());
                }
            }
            KeyCode::Char('y') => {
                if let Some(ref color) = self.current {
                    self.status_message = Some(format!("Copied: {}", color.hex));
                }
            }
            _ => {}
        }
        false
    }

    fn parse_input(&mut self) {
        self.current = match self.format {
            ColorFormat::Hex => ColorValues::from_hex(&self.input),
            ColorFormat::Rgb => self.parse_rgb(),
            _ => None,
        };
    }

    fn parse_rgb(&self) -> Option<ColorValues> {
        let parts: Vec<&str> = self.input.split(|c| c == ',' || c == ' ')
            .filter(|s| !s.is_empty())
            .collect();

        if parts.len() != 3 {
            return None;
        }

        let r = parts[0].parse().ok()?;
        let g = parts[1].parse().ok()?;
        let b = parts[2].parse().ok()?;

        Some(ColorValues::from_rgb(r, g, b))
    }

    fn save_current(&mut self) {
        if let Some(ref color) = self.current {
            let name = format!("Color {}", self.saved.len() + 1);
            self.saved.push(SavedColor {
                name,
                values: color.clone(),
            });
            self.status_message = Some("Color saved!".to_string());
        }
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }
        "Tab:format j/k:saved Enter:load Ctrl+S:save d:delete y:copy q:quit".to_string()
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
