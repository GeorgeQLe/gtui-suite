use anyhow::Result;
use rusqlite::{Connection, types::Value};

pub struct Database {
    conn: Connection,
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct TableInfo {
    pub name: String,
    pub row_count: usize,
}

#[derive(Debug, Clone)]
pub struct ColumnInfo {
    pub name: String,
    pub col_type: String,
    pub nullable: bool,
    pub primary_key: bool,
}

#[derive(Debug, Clone)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub affected_rows: Option<usize>,
}

impl Database {
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        Ok(Self {
            conn,
            path: path.to_string(),
        })
    }

    pub fn list_tables(&self) -> Result<Vec<TableInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name"
        )?;

        let tables: Vec<String> = stmt
            .query_map([], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        let mut result = Vec::new();
        for name in tables {
            let count = self.get_row_count(&name).unwrap_or(0);
            result.push(TableInfo { name, row_count: count });
        }

        Ok(result)
    }

    fn get_row_count(&self, table: &str) -> Result<usize> {
        let sql = format!("SELECT COUNT(*) FROM \"{}\"", table.replace('"', "\"\""));
        let count: i64 = self.conn.query_row(&sql, [], |row| row.get(0))?;
        Ok(count as usize)
    }

    pub fn get_table_schema(&self, table: &str) -> Result<Vec<ColumnInfo>> {
        let sql = format!("PRAGMA table_info(\"{}\")", table.replace('"', "\"\""));
        let mut stmt = self.conn.prepare(&sql)?;

        let columns = stmt
            .query_map([], |row| {
                Ok(ColumnInfo {
                    name: row.get(1)?,
                    col_type: row.get(2)?,
                    nullable: row.get::<_, i32>(3)? == 0,
                    primary_key: row.get::<_, i32>(5)? == 1,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(columns)
    }

    pub fn query(&self, sql: &str) -> Result<QueryResult> {
        let sql_trimmed = sql.trim().to_uppercase();

        // Check if it's a SELECT or other read query
        if sql_trimmed.starts_with("SELECT") ||
           sql_trimmed.starts_with("PRAGMA") ||
           sql_trimmed.starts_with("EXPLAIN")
        {
            self.execute_query(sql)
        } else {
            self.execute_statement(sql)
        }
    }

    fn execute_query(&self, sql: &str) -> Result<QueryResult> {
        let mut stmt = self.conn.prepare(sql)?;

        let columns: Vec<String> = stmt
            .column_names()
            .iter()
            .map(|s| s.to_string())
            .collect();

        let rows: Vec<Vec<String>> = stmt
            .query_map([], |row| {
                let mut values = Vec::new();
                for i in 0..columns.len() {
                    let value: Value = row.get(i)?;
                    let s = match value {
                        Value::Null => "NULL".to_string(),
                        Value::Integer(i) => i.to_string(),
                        Value::Real(f) => f.to_string(),
                        Value::Text(s) => s,
                        Value::Blob(b) => format!("<BLOB {} bytes>", b.len()),
                    };
                    values.push(s);
                }
                Ok(values)
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(QueryResult {
            columns,
            rows,
            affected_rows: None,
        })
    }

    fn execute_statement(&self, sql: &str) -> Result<QueryResult> {
        let affected = self.conn.execute(sql, [])?;

        Ok(QueryResult {
            columns: vec!["Result".to_string()],
            rows: vec![vec![format!("{} row(s) affected", affected)]],
            affected_rows: Some(affected),
        })
    }

    pub fn get_table_data(&self, table: &str, limit: usize, offset: usize) -> Result<QueryResult> {
        let sql = format!(
            "SELECT * FROM \"{}\" LIMIT {} OFFSET {}",
            table.replace('"', "\"\""),
            limit,
            offset
        );
        self.execute_query(&sql)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_open_memory() {
        let db = Database::open(":memory:");
        assert!(db.is_ok());
    }

    #[test]
    fn test_list_tables() {
        let db = Database::open(":memory:").unwrap();
        db.conn.execute("CREATE TABLE test (id INTEGER PRIMARY KEY)", []).unwrap();

        let tables = db.list_tables().unwrap();
        assert_eq!(tables.len(), 1);
        assert_eq!(tables[0].name, "test");
    }

    #[test]
    fn test_query() {
        let db = Database::open(":memory:").unwrap();
        db.conn.execute("CREATE TABLE test (id INTEGER, name TEXT)", []).unwrap();
        db.conn.execute("INSERT INTO test VALUES (1, 'Alice')", []).unwrap();

        let result = db.query("SELECT * FROM test").unwrap();
        assert_eq!(result.columns, vec!["id", "name"]);
        assert_eq!(result.rows.len(), 1);
    }
}
