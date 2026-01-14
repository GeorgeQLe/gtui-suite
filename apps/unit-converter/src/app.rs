use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Length,
    Weight,
    Temperature,
    Data,
    Time,
}

impl Category {
    pub fn name(&self) -> &'static str {
        match self {
            Category::Length => "Length",
            Category::Weight => "Weight",
            Category::Temperature => "Temperature",
            Category::Data => "Data",
            Category::Time => "Time",
        }
    }

    pub fn all() -> Vec<Category> {
        vec![
            Category::Length,
            Category::Weight,
            Category::Temperature,
            Category::Data,
            Category::Time,
        ]
    }

    pub fn units(&self) -> Vec<(&'static str, &'static str, f64)> {
        // (short, long, factor_to_base)
        match self {
            Category::Length => vec![
                ("mm", "Millimeters", 0.001),
                ("cm", "Centimeters", 0.01),
                ("m", "Meters", 1.0),
                ("km", "Kilometers", 1000.0),
                ("in", "Inches", 0.0254),
                ("ft", "Feet", 0.3048),
                ("yd", "Yards", 0.9144),
                ("mi", "Miles", 1609.34),
            ],
            Category::Weight => vec![
                ("mg", "Milligrams", 0.001),
                ("g", "Grams", 1.0),
                ("kg", "Kilograms", 1000.0),
                ("oz", "Ounces", 28.3495),
                ("lb", "Pounds", 453.592),
                ("t", "Metric Tons", 1000000.0),
            ],
            Category::Temperature => vec![
                ("C", "Celsius", 1.0),
                ("F", "Fahrenheit", 1.0),
                ("K", "Kelvin", 1.0),
            ],
            Category::Data => vec![
                ("B", "Bytes", 1.0),
                ("KB", "Kilobytes", 1024.0),
                ("MB", "Megabytes", 1048576.0),
                ("GB", "Gigabytes", 1073741824.0),
                ("TB", "Terabytes", 1099511627776.0),
                ("KiB", "Kibibytes", 1024.0),
                ("MiB", "Mebibytes", 1048576.0),
                ("GiB", "Gibibytes", 1073741824.0),
            ],
            Category::Time => vec![
                ("ms", "Milliseconds", 0.001),
                ("s", "Seconds", 1.0),
                ("min", "Minutes", 60.0),
                ("h", "Hours", 3600.0),
                ("d", "Days", 86400.0),
                ("w", "Weeks", 604800.0),
            ],
        }
    }
}

pub struct App {
    pub category: Category,
    pub input: String,
    pub from_unit: usize,
    pub results: Vec<(String, String, f64)>,
    pub selected: usize,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            category: Category::Length,
            input: "1".to_string(),
            from_unit: 2, // meters
            results: Vec::new(),
            selected: 0,
            status_message: None,
        };
        app.convert();
        app
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return true;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Tab => {
                let cats = Category::all();
                let idx = cats.iter().position(|c| *c == self.category).unwrap_or(0);
                self.category = cats[(idx + 1) % cats.len()];
                self.from_unit = 0;
                self.convert();
            }
            KeyCode::BackTab => {
                let cats = Category::all();
                let idx = cats.iter().position(|c| *c == self.category).unwrap_or(0);
                self.category = cats[(idx + cats.len() - 1) % cats.len()];
                self.from_unit = 0;
                self.convert();
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected < self.results.len().saturating_sub(1) {
                    self.selected += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
            }
            KeyCode::Char('u') => {
                let units = self.category.units();
                self.from_unit = (self.from_unit + 1) % units.len();
                self.convert();
            }
            KeyCode::Backspace => {
                self.input.pop();
                self.convert();
            }
            KeyCode::Char(c) if c.is_ascii_digit() || c == '.' || c == '-' => {
                self.input.push(c);
                self.convert();
            }
            KeyCode::Char('y') => {
                if let Some((_, _, value)) = self.results.get(self.selected) {
                    self.status_message = Some(format!("Copied: {}", value));
                }
            }
            _ => {}
        }
        false
    }

    fn convert(&mut self) {
        self.results.clear();

        let value: f64 = match self.input.parse() {
            Ok(v) => v,
            Err(_) => {
                self.status_message = Some("Invalid number".to_string());
                return;
            }
        };

        let units = self.category.units();

        if self.category == Category::Temperature {
            self.convert_temperature(value);
            return;
        }

        let from_factor = units.get(self.from_unit).map(|u| u.2).unwrap_or(1.0);
        let base_value = value * from_factor;

        for (short, long, factor) in units {
            let result = base_value / factor;
            self.results.push((short.to_string(), long.to_string(), result));
        }

        self.selected = self.selected.min(self.results.len().saturating_sub(1));
    }

    fn convert_temperature(&mut self, value: f64) {
        let units = self.category.units();
        let from = units.get(self.from_unit).map(|u| u.0).unwrap_or("C");

        // Convert to Celsius first
        let celsius = match from {
            "C" => value,
            "F" => (value - 32.0) * 5.0 / 9.0,
            "K" => value - 273.15,
            _ => value,
        };

        self.results.push(("C".to_string(), "Celsius".to_string(), celsius));
        self.results.push(("F".to_string(), "Fahrenheit".to_string(), celsius * 9.0 / 5.0 + 32.0));
        self.results.push(("K".to_string(), "Kelvin".to_string(), celsius + 273.15));
    }

    pub fn current_unit(&self) -> &str {
        self.category.units().get(self.from_unit).map(|u| u.0).unwrap_or("")
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }
        "Tab:category u:unit j/k:nav y:copy q:quit".to_string()
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
