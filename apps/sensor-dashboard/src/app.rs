use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone)]
pub enum SensorType { Temperature, Voltage, Fan, Power, Current }

#[derive(Debug, Clone)]
pub struct Sensor {
    pub name: String,
    pub sensor_type: SensorType,
    pub value: f32,
    pub unit: String,
    pub min: f32,
    pub max: f32,
    pub critical: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct SensorChip {
    pub name: String,
    pub adapter: String,
    pub sensors: Vec<Sensor>,
}

pub struct App {
    pub chips: Vec<SensorChip>,
    pub selected_chip: usize,
    pub selected_sensor: usize,
    pub tick_count: u64,
}

impl App {
    pub fn new() -> Self {
        Self {
            chips: vec![
                SensorChip {
                    name: "coretemp-isa-0000".into(),
                    adapter: "ISA adapter".into(),
                    sensors: vec![
                        Sensor { name: "Package id 0".into(), sensor_type: SensorType::Temperature, value: 52.0, unit: "C".into(), min: 20.0, max: 100.0, critical: Some(100.0) },
                        Sensor { name: "Core 0".into(), sensor_type: SensorType::Temperature, value: 50.0, unit: "C".into(), min: 20.0, max: 100.0, critical: Some(100.0) },
                        Sensor { name: "Core 1".into(), sensor_type: SensorType::Temperature, value: 51.0, unit: "C".into(), min: 20.0, max: 100.0, critical: Some(100.0) },
                        Sensor { name: "Core 2".into(), sensor_type: SensorType::Temperature, value: 49.0, unit: "C".into(), min: 20.0, max: 100.0, critical: Some(100.0) },
                        Sensor { name: "Core 3".into(), sensor_type: SensorType::Temperature, value: 52.0, unit: "C".into(), min: 20.0, max: 100.0, critical: Some(100.0) },
                    ],
                },
                SensorChip {
                    name: "nct6798-isa-0290".into(),
                    adapter: "ISA adapter".into(),
                    sensors: vec![
                        Sensor { name: "Vcore".into(), sensor_type: SensorType::Voltage, value: 1.1, unit: "V".into(), min: 0.0, max: 2.0, critical: None },
                        Sensor { name: "+3.3V".into(), sensor_type: SensorType::Voltage, value: 3.31, unit: "V".into(), min: 0.0, max: 4.0, critical: None },
                        Sensor { name: "+5V".into(), sensor_type: SensorType::Voltage, value: 5.02, unit: "V".into(), min: 0.0, max: 6.0, critical: None },
                        Sensor { name: "+12V".into(), sensor_type: SensorType::Voltage, value: 12.1, unit: "V".into(), min: 0.0, max: 15.0, critical: None },
                        Sensor { name: "fan1".into(), sensor_type: SensorType::Fan, value: 1200.0, unit: "RPM".into(), min: 0.0, max: 5000.0, critical: None },
                        Sensor { name: "fan2".into(), sensor_type: SensorType::Fan, value: 850.0, unit: "RPM".into(), min: 0.0, max: 3000.0, critical: None },
                    ],
                },
                SensorChip {
                    name: "acpitz-acpi-0".into(),
                    adapter: "ACPI interface".into(),
                    sensors: vec![
                        Sensor { name: "temp1".into(), sensor_type: SensorType::Temperature, value: 45.0, unit: "C".into(), min: 0.0, max: 110.0, critical: Some(110.0) },
                    ],
                },
            ],
            selected_chip: 0,
            selected_sensor: 0,
            tick_count: 0,
        }
    }

    pub fn tick(&mut self) {
        self.tick_count += 1;
        use std::time::{SystemTime, UNIX_EPOCH};
        let secs = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        // Simulate sensor value changes
        for chip in &mut self.chips {
            for sensor in &mut chip.sensors {
                match sensor.sensor_type {
                    SensorType::Temperature => {
                        let base = (sensor.max - sensor.min) * 0.4 + sensor.min;
                        sensor.value = base + ((secs % 10) as f32 * 0.5);
                    },
                    SensorType::Fan => {
                        sensor.value = sensor.value * 0.95 + (800.0 + (secs % 20) as f32 * 30.0) * 0.05;
                    },
                    _ => {}
                }
            }
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') { return true; }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char('j') | KeyCode::Down => {
                if let Some(chip) = self.chips.get(self.selected_chip) {
                    if self.selected_sensor < chip.sensors.len().saturating_sub(1) {
                        self.selected_sensor += 1;
                    }
                }
            },
            KeyCode::Char('k') | KeyCode::Up => self.selected_sensor = self.selected_sensor.saturating_sub(1),
            KeyCode::Tab | KeyCode::Char('l') | KeyCode::Right => {
                self.selected_chip = (self.selected_chip + 1) % self.chips.len().max(1);
                self.selected_sensor = 0;
            },
            KeyCode::BackTab | KeyCode::Char('h') | KeyCode::Left => {
                self.selected_chip = self.selected_chip.checked_sub(1).unwrap_or(self.chips.len().saturating_sub(1));
                self.selected_sensor = 0;
            },
            _ => {}
        }
        false
    }

    pub fn status_text(&self) -> String {
        "h/l:chip j/k:sensor q:quit".into()
    }
}

impl Default for App { fn default() -> Self { Self::new() } }
