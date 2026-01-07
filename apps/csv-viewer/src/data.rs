use anyhow::Result;
use std::cmp::Ordering;
use std::fs::File;
use std::path::PathBuf;

pub struct CsvData {
    pub path: Option<PathBuf>,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub delimiter: u8,
}

impl CsvData {
    pub fn new() -> Self {
        Self {
            path: None,
            headers: Vec::new(),
            rows: Vec::new(),
            delimiter: b',',
        }
    }

    pub fn load(path: PathBuf) -> Result<Self> {
        let delimiter = if path.extension().map(|e| e == "tsv").unwrap_or(false) {
            b'\t'
        } else {
            b','
        };

        let file = File::open(&path)?;
        let mut reader = csv::ReaderBuilder::new()
            .delimiter(delimiter)
            .has_headers(true)
            .flexible(true)
            .from_reader(file);

        let headers: Vec<String> = reader.headers()?
            .iter()
            .map(|s| s.to_string())
            .collect();

        let mut rows = Vec::new();
        for result in reader.records() {
            let record = result?;
            let row: Vec<String> = record.iter().map(|s| s.to_string()).collect();
            rows.push(row);
        }

        Ok(Self {
            path: Some(path),
            headers,
            rows,
            delimiter,
        })
    }

    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    pub fn col_count(&self) -> usize {
        self.headers.len()
    }

    pub fn get_cell(&self, row: usize, col: usize) -> Option<&str> {
        self.rows.get(row).and_then(|r| r.get(col).map(|s| s.as_str()))
    }

    pub fn sort_by_column(&mut self, col: usize, ascending: bool) {
        self.rows.sort_by(|a, b| {
            let val_a = a.get(col).map(|s| s.as_str()).unwrap_or("");
            let val_b = b.get(col).map(|s| s.as_str()).unwrap_or("");

            // Try numeric comparison first
            let cmp = match (val_a.parse::<f64>(), val_b.parse::<f64>()) {
                (Ok(num_a), Ok(num_b)) => num_a.partial_cmp(&num_b).unwrap_or(Ordering::Equal),
                _ => val_a.cmp(val_b),
            };

            if ascending { cmp } else { cmp.reverse() }
        });
    }

    pub fn filter(&self, column: usize, query: &str) -> Vec<usize> {
        let query_lower = query.to_lowercase();
        self.rows.iter()
            .enumerate()
            .filter(|(_, row)| {
                row.get(column)
                    .map(|cell| cell.to_lowercase().contains(&query_lower))
                    .unwrap_or(false)
            })
            .map(|(i, _)| i)
            .collect()
    }

    pub fn search(&self, query: &str) -> Vec<(usize, usize)> {
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        for (row_idx, row) in self.rows.iter().enumerate() {
            for (col_idx, cell) in row.iter().enumerate() {
                if cell.to_lowercase().contains(&query_lower) {
                    results.push((row_idx, col_idx));
                }
            }
        }

        results
    }
}

pub fn calculate_column_widths(data: &CsvData, max_width: u16) -> Vec<u16> {
    let col_count = data.col_count();
    if col_count == 0 {
        return Vec::new();
    }

    let mut widths: Vec<u16> = data.headers.iter()
        .map(|h| h.len() as u16 + 2)
        .collect();

    for row in &data.rows {
        for (i, cell) in row.iter().enumerate() {
            if i < widths.len() {
                widths[i] = widths[i].max(cell.len() as u16 + 2).min(50);
            }
        }
    }

    // Scale if total exceeds max
    let total: u16 = widths.iter().sum();
    if total > max_width {
        let scale = max_width as f32 / total as f32;
        for w in &mut widths {
            *w = ((*w as f32 * scale) as u16).max(5);
        }
    }

    widths
}
