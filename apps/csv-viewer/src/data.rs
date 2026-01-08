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

#[cfg(test)]
mod tests {
    use super::*;

    fn test_data() -> CsvData {
        CsvData {
            path: None,
            headers: vec!["Name".to_string(), "Age".to_string(), "City".to_string()],
            rows: vec![
                vec!["Alice".to_string(), "30".to_string(), "New York".to_string()],
                vec!["Bob".to_string(), "25".to_string(), "Los Angeles".to_string()],
                vec!["Charlie".to_string(), "35".to_string(), "Chicago".to_string()],
            ],
            delimiter: b',',
        }
    }

    #[test]
    fn test_new() {
        let data = CsvData::new();
        assert_eq!(data.row_count(), 0);
        assert_eq!(data.col_count(), 0);
    }

    #[test]
    fn test_row_col_count() {
        let data = test_data();
        assert_eq!(data.row_count(), 3);
        assert_eq!(data.col_count(), 3);
    }

    #[test]
    fn test_get_cell() {
        let data = test_data();
        assert_eq!(data.get_cell(0, 0), Some("Alice"));
        assert_eq!(data.get_cell(1, 2), Some("Los Angeles"));
        assert_eq!(data.get_cell(10, 0), None);
        assert_eq!(data.get_cell(0, 10), None);
    }

    #[test]
    fn test_sort_by_column_string() {
        let mut data = test_data();
        data.sort_by_column(0, true);
        assert_eq!(data.get_cell(0, 0), Some("Alice"));
        assert_eq!(data.get_cell(2, 0), Some("Charlie"));

        data.sort_by_column(0, false);
        assert_eq!(data.get_cell(0, 0), Some("Charlie"));
        assert_eq!(data.get_cell(2, 0), Some("Alice"));
    }

    #[test]
    fn test_sort_by_column_numeric() {
        let mut data = test_data();
        data.sort_by_column(1, true);
        assert_eq!(data.get_cell(0, 1), Some("25"));
        assert_eq!(data.get_cell(2, 1), Some("35"));

        data.sort_by_column(1, false);
        assert_eq!(data.get_cell(0, 1), Some("35"));
        assert_eq!(data.get_cell(2, 1), Some("25"));
    }

    #[test]
    fn test_filter() {
        let data = test_data();
        let results = data.filter(2, "Los");
        assert_eq!(results, vec![1]);

        let results = data.filter(2, "york");
        assert_eq!(results, vec![0]);

        let results = data.filter(0, "xyz");
        assert!(results.is_empty());
    }

    #[test]
    fn test_search() {
        let data = test_data();
        let results = data.search("charlie");
        assert_eq!(results, vec![(2, 0)]);

        let results = data.search("25");
        assert_eq!(results, vec![(1, 1)]);

        let results = data.search("xyz");
        assert!(results.is_empty());
    }

    #[test]
    fn test_calculate_column_widths() {
        let data = test_data();
        let widths = calculate_column_widths(&data, 100);
        assert_eq!(widths.len(), 3);
        assert!(widths.iter().all(|&w| w >= 5));
    }

    #[test]
    fn test_calculate_column_widths_empty() {
        let data = CsvData::new();
        let widths = calculate_column_widths(&data, 100);
        assert!(widths.is_empty());
    }
}
