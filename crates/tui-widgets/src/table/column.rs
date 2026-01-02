//! Column definitions for DataTable.

use super::CellContent;

/// Column width specification.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColumnWidth {
    /// Fixed width in characters
    Fixed(u16),
    /// Percentage of available width
    Percentage(f32),
    /// Flexible width with relative weight
    Flex(u16),
}

impl Default for ColumnWidth {
    fn default() -> Self {
        Self::Flex(1)
    }
}

/// Aggregate function for column footer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AggregateFunc {
    /// Sum of numeric values
    Sum,
    /// Average of numeric values
    Avg,
    /// Minimum value
    Min,
    /// Maximum value
    Max,
    /// Count of rows
    Count,
}

impl AggregateFunc {
    /// Calculate the aggregate for a set of values.
    pub fn calculate(&self, values: &[f64]) -> f64 {
        if values.is_empty() {
            return 0.0;
        }

        match self {
            Self::Sum => values.iter().sum(),
            Self::Avg => values.iter().sum::<f64>() / values.len() as f64,
            Self::Min => values.iter().cloned().fold(f64::INFINITY, f64::min),
            Self::Max => values.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
            Self::Count => values.len() as f64,
        }
    }

    /// Get a display label for the aggregate.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Sum => "Sum",
            Self::Avg => "Avg",
            Self::Min => "Min",
            Self::Max => "Max",
            Self::Count => "Count",
        }
    }
}

/// Column definition for DataTable.
pub struct Column<T> {
    /// Header text
    pub header: String,
    /// Width specification
    pub width: ColumnWidth,
    /// Function to extract cell content from row
    pub accessor: fn(&T) -> CellContent,
    /// Whether this column is sortable
    pub sortable: bool,
    /// Whether this column is filterable
    pub filterable: bool,
    /// Optional aggregate function for footer
    pub aggregate: Option<AggregateFunc>,
    /// Whether this column is resizable
    pub resizable: bool,
}

impl<T> Column<T> {
    /// Create a new column with header and accessor.
    pub fn new(header: impl Into<String>, accessor: fn(&T) -> CellContent) -> Self {
        Self {
            header: header.into(),
            width: ColumnWidth::default(),
            accessor,
            sortable: false,
            filterable: true,
            aggregate: None,
            resizable: false,
        }
    }

    /// Set the column width.
    pub fn width(mut self, width: ColumnWidth) -> Self {
        self.width = width;
        self
    }

    /// Set whether the column is sortable.
    pub fn sortable(mut self, sortable: bool) -> Self {
        self.sortable = sortable;
        self
    }

    /// Set whether the column is filterable.
    pub fn filterable(mut self, filterable: bool) -> Self {
        self.filterable = filterable;
        self
    }

    /// Set the aggregate function.
    pub fn aggregate(mut self, func: AggregateFunc) -> Self {
        self.aggregate = Some(func);
        self
    }

    /// Set whether the column is resizable.
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregate_sum() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(AggregateFunc::Sum.calculate(&values), 15.0);
    }

    #[test]
    fn test_aggregate_avg() {
        let values = vec![2.0, 4.0, 6.0];
        assert_eq!(AggregateFunc::Avg.calculate(&values), 4.0);
    }

    #[test]
    fn test_aggregate_minmax() {
        let values = vec![5.0, 2.0, 8.0, 1.0];
        assert_eq!(AggregateFunc::Min.calculate(&values), 1.0);
        assert_eq!(AggregateFunc::Max.calculate(&values), 8.0);
    }

    #[test]
    fn test_aggregate_count() {
        let values = vec![1.0, 2.0, 3.0];
        assert_eq!(AggregateFunc::Count.calculate(&values), 3.0);
    }

    #[test]
    fn test_aggregate_empty() {
        let values: Vec<f64> = vec![];
        assert_eq!(AggregateFunc::Sum.calculate(&values), 0.0);
        assert_eq!(AggregateFunc::Count.calculate(&values), 0.0);
    }

    #[test]
    fn test_column_builder() {
        struct Row {
            name: String,
        }

        let col = Column::new("Name", |r: &Row| r.name.clone().into())
            .width(ColumnWidth::Fixed(20))
            .sortable(true)
            .aggregate(AggregateFunc::Count);

        assert_eq!(col.header, "Name");
        assert_eq!(col.width, ColumnWidth::Fixed(20));
        assert!(col.sortable);
        assert_eq!(col.aggregate, Some(AggregateFunc::Count));
    }
}
