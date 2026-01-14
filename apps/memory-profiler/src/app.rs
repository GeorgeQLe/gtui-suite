use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone)]
pub struct MemoryRegion {
    pub name: String,
    pub category: MemoryCategory,
    pub size_bytes: u64,
    pub resident_bytes: u64,
    pub shared_bytes: u64,
    pub private_bytes: u64,
    pub permission: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryCategory {
    Heap,
    Stack,
    Code,
    Data,
    SharedLib,
    MappedFile,
    Anonymous,
}

impl MemoryCategory {
    pub fn name(&self) -> &'static str {
        match self {
            MemoryCategory::Heap => "Heap",
            MemoryCategory::Stack => "Stack",
            MemoryCategory::Code => "Code",
            MemoryCategory::Data => "Data",
            MemoryCategory::SharedLib => "Shared",
            MemoryCategory::MappedFile => "Mapped",
            MemoryCategory::Anonymous => "Anon",
        }
    }
}

#[derive(Debug, Clone)]
pub struct MemorySample {
    pub timestamp: u64,
    pub heap_mb: f64,
    pub rss_mb: f64,
}

#[derive(Debug, Clone)]
pub struct AllocationSite {
    pub function: String,
    pub file: String,
    pub line: usize,
    pub alloc_count: u64,
    pub total_bytes: u64,
    pub live_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Overview,
    Regions,
    Allocations,
    Timeline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortBy {
    Size,
    Resident,
    Private,
    Name,
}

pub struct App {
    pub regions: Vec<MemoryRegion>,
    pub allocations: Vec<AllocationSite>,
    pub samples: Vec<MemorySample>,
    pub selected: usize,
    pub view_mode: ViewMode,
    pub sort_by: SortBy,
    pub is_recording: bool,
    pub tick_count: u64,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            regions: create_demo_regions(),
            allocations: create_demo_allocations(),
            samples: create_demo_samples(),
            selected: 0,
            view_mode: ViewMode::Overview,
            sort_by: SortBy::Size,
            is_recording: false,
            tick_count: 0,
            status_message: None,
        }
    }

    pub fn tick(&mut self) {
        self.tick_count += 1;

        if self.is_recording && self.tick_count % 10 == 0 {
            let base = 150.0 + (self.tick_count as f64 * 0.05).sin() * 30.0;
            self.samples.push(MemorySample {
                timestamp: self.tick_count,
                heap_mb: base,
                rss_mb: base * 1.3,
            });

            if self.samples.len() > 100 {
                self.samples.remove(0);
            }
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return true;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char('j') | KeyCode::Down => {
                let max = match self.view_mode {
                    ViewMode::Regions => self.regions.len(),
                    ViewMode::Allocations => self.allocations.len(),
                    _ => 0,
                };
                if max > 0 && self.selected < max - 1 {
                    self.selected += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
            }
            KeyCode::Char('1') => {
                self.view_mode = ViewMode::Overview;
                self.selected = 0;
                self.status_message = Some("View: Overview".to_string());
            }
            KeyCode::Char('2') => {
                self.view_mode = ViewMode::Regions;
                self.selected = 0;
                self.status_message = Some("View: Memory Regions".to_string());
            }
            KeyCode::Char('3') => {
                self.view_mode = ViewMode::Allocations;
                self.selected = 0;
                self.status_message = Some("View: Allocation Sites".to_string());
            }
            KeyCode::Char('4') => {
                self.view_mode = ViewMode::Timeline;
                self.selected = 0;
                self.status_message = Some("View: Timeline".to_string());
            }
            KeyCode::Char('s') => {
                self.sort_by = SortBy::Size;
                self.sort_data();
                self.status_message = Some("Sorted by size".to_string());
            }
            KeyCode::Char('p') => {
                self.sort_by = SortBy::Private;
                self.sort_data();
                self.status_message = Some("Sorted by private memory".to_string());
            }
            KeyCode::Char('n') => {
                self.sort_by = SortBy::Name;
                self.sort_data();
                self.status_message = Some("Sorted by name".to_string());
            }
            KeyCode::Char('r') => {
                self.is_recording = !self.is_recording;
                self.status_message = Some(if self.is_recording {
                    "Recording started".to_string()
                } else {
                    "Recording stopped".to_string()
                });
            }
            KeyCode::Char('c') => {
                self.samples.clear();
                self.status_message = Some("Samples cleared".to_string());
            }
            _ => {}
        }
        false
    }

    fn sort_data(&mut self) {
        match self.sort_by {
            SortBy::Size => {
                self.regions.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
                self.allocations.sort_by(|a, b| b.total_bytes.cmp(&a.total_bytes));
            }
            SortBy::Resident => {
                self.regions.sort_by(|a, b| b.resident_bytes.cmp(&a.resident_bytes));
            }
            SortBy::Private => {
                self.regions.sort_by(|a, b| b.private_bytes.cmp(&a.private_bytes));
            }
            SortBy::Name => {
                self.regions.sort_by(|a, b| a.name.cmp(&b.name));
                self.allocations.sort_by(|a, b| a.function.cmp(&b.function));
            }
        }
    }

    pub fn total_memory(&self) -> u64 {
        self.regions.iter().map(|r| r.size_bytes).sum()
    }

    pub fn total_resident(&self) -> u64 {
        self.regions.iter().map(|r| r.resident_bytes).sum()
    }

    pub fn heap_memory(&self) -> u64 {
        self.regions
            .iter()
            .filter(|r| r.category == MemoryCategory::Heap)
            .map(|r| r.size_bytes)
            .sum()
    }

    pub fn current_heap_mb(&self) -> f64 {
        self.samples.last().map(|s| s.heap_mb).unwrap_or(0.0)
    }

    pub fn current_rss_mb(&self) -> f64 {
        self.samples.last().map(|s| s.rss_mb).unwrap_or(0.0)
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }
        let rec = if self.is_recording { "[REC]" } else { "" };
        format!("j/k:nav 1-4:view s:size p:private n:name r:record c:clear q:quit {}", rec)
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn create_demo_regions() -> Vec<MemoryRegion> {
    vec![
        MemoryRegion {
            name: "[heap]".to_string(),
            category: MemoryCategory::Heap,
            size_bytes: 150_000_000,
            resident_bytes: 120_000_000,
            shared_bytes: 0,
            private_bytes: 120_000_000,
            permission: "rw-p".to_string(),
        },
        MemoryRegion {
            name: "[stack]".to_string(),
            category: MemoryCategory::Stack,
            size_bytes: 8_000_000,
            resident_bytes: 500_000,
            shared_bytes: 0,
            private_bytes: 500_000,
            permission: "rw-p".to_string(),
        },
        MemoryRegion {
            name: "libc.so.6".to_string(),
            category: MemoryCategory::SharedLib,
            size_bytes: 2_000_000,
            resident_bytes: 1_500_000,
            shared_bytes: 1_500_000,
            private_bytes: 0,
            permission: "r-xp".to_string(),
        },
        MemoryRegion {
            name: "libpthread.so.0".to_string(),
            category: MemoryCategory::SharedLib,
            size_bytes: 150_000,
            resident_bytes: 100_000,
            shared_bytes: 100_000,
            private_bytes: 0,
            permission: "r-xp".to_string(),
        },
        MemoryRegion {
            name: "app_binary".to_string(),
            category: MemoryCategory::Code,
            size_bytes: 5_000_000,
            resident_bytes: 4_000_000,
            shared_bytes: 0,
            private_bytes: 4_000_000,
            permission: "r-xp".to_string(),
        },
        MemoryRegion {
            name: "[anon:mmap]".to_string(),
            category: MemoryCategory::Anonymous,
            size_bytes: 50_000_000,
            resident_bytes: 45_000_000,
            shared_bytes: 0,
            private_bytes: 45_000_000,
            permission: "rw-p".to_string(),
        },
        MemoryRegion {
            name: "data.db".to_string(),
            category: MemoryCategory::MappedFile,
            size_bytes: 100_000_000,
            resident_bytes: 30_000_000,
            shared_bytes: 30_000_000,
            private_bytes: 0,
            permission: "r--s".to_string(),
        },
    ]
}

fn create_demo_allocations() -> Vec<AllocationSite> {
    vec![
        AllocationSite {
            function: "process_request".to_string(),
            file: "src/handler.rs".to_string(),
            line: 45,
            alloc_count: 100000,
            total_bytes: 50_000_000,
            live_bytes: 10_000_000,
        },
        AllocationSite {
            function: "parse_json".to_string(),
            file: "src/parser.rs".to_string(),
            line: 120,
            alloc_count: 500000,
            total_bytes: 25_000_000,
            live_bytes: 5_000_000,
        },
        AllocationSite {
            function: "cache_insert".to_string(),
            file: "src/cache.rs".to_string(),
            line: 88,
            alloc_count: 50000,
            total_bytes: 40_000_000,
            live_bytes: 40_000_000,
        },
        AllocationSite {
            function: "create_response".to_string(),
            file: "src/response.rs".to_string(),
            line: 33,
            alloc_count: 100000,
            total_bytes: 20_000_000,
            live_bytes: 2_000_000,
        },
        AllocationSite {
            function: "log_entry".to_string(),
            file: "src/logging.rs".to_string(),
            line: 15,
            alloc_count: 1000000,
            total_bytes: 10_000_000,
            live_bytes: 1_000_000,
        },
    ]
}

fn create_demo_samples() -> Vec<MemorySample> {
    (0..50)
        .map(|i| {
            let base = 150.0 + (i as f64 * 0.1).sin() * 20.0;
            MemorySample {
                timestamp: i,
                heap_mb: base,
                rss_mb: base * 1.3,
            }
        })
        .collect()
}
