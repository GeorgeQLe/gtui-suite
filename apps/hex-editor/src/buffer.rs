use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use anyhow::Result;

pub struct HexBuffer {
    pub path: Option<PathBuf>,
    pub data: Vec<u8>,
    pub modified: bool,
    pub undo_stack: Vec<EditOperation>,
    pub redo_stack: Vec<EditOperation>,
}

#[derive(Clone)]
pub struct EditOperation {
    pub offset: usize,
    pub old_value: u8,
    pub new_value: u8,
}

impl HexBuffer {
    pub fn new() -> Self {
        Self {
            path: None,
            data: Vec::new(),
            modified: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn open(path: PathBuf) -> Result<Self> {
        let mut file = File::open(&path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        Ok(Self {
            path: Some(path),
            data,
            modified: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        })
    }

    pub fn save(&mut self) -> Result<()> {
        if let Some(path) = &self.path {
            let mut file = File::create(path)?;
            file.write_all(&self.data)?;
            self.modified = false;
            Ok(())
        } else {
            Err(anyhow::anyhow!("No file path set"))
        }
    }

    pub fn save_as(&mut self, path: PathBuf) -> Result<()> {
        let mut file = File::create(&path)?;
        file.write_all(&self.data)?;
        self.path = Some(path);
        self.modified = false;
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn get(&self, offset: usize) -> Option<u8> {
        self.data.get(offset).copied()
    }

    pub fn set(&mut self, offset: usize, value: u8) {
        if offset < self.data.len() {
            let old_value = self.data[offset];
            if old_value != value {
                self.undo_stack.push(EditOperation {
                    offset,
                    old_value,
                    new_value: value,
                });
                self.redo_stack.clear();
                self.data[offset] = value;
                self.modified = true;
            }
        }
    }

    pub fn insert(&mut self, offset: usize, value: u8) {
        let offset = offset.min(self.data.len());
        self.data.insert(offset, value);
        self.modified = true;
    }

    pub fn delete(&mut self, offset: usize) {
        if offset < self.data.len() {
            self.data.remove(offset);
            self.modified = true;
        }
    }

    pub fn undo(&mut self) -> bool {
        if let Some(op) = self.undo_stack.pop() {
            self.data[op.offset] = op.old_value;
            self.redo_stack.push(op);
            self.modified = true;
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if let Some(op) = self.redo_stack.pop() {
            self.data[op.offset] = op.new_value;
            self.undo_stack.push(op);
            self.modified = true;
            true
        } else {
            false
        }
    }

    pub fn search(&self, pattern: &[u8], start: usize) -> Option<usize> {
        if pattern.is_empty() || start >= self.data.len() {
            return None;
        }

        for i in start..self.data.len().saturating_sub(pattern.len() - 1) {
            if self.data[i..].starts_with(pattern) {
                return Some(i);
            }
        }
        None
    }

    pub fn search_hex(&self, hex_str: &str, start: usize) -> Option<usize> {
        let pattern = parse_hex_string(hex_str)?;
        self.search(&pattern, start)
    }
}

pub fn parse_hex_string(s: &str) -> Option<Vec<u8>> {
    let clean: String = s.chars().filter(|c| c.is_ascii_hexdigit()).collect();
    if clean.len() % 2 != 0 {
        return None;
    }

    let mut result = Vec::new();
    for i in (0..clean.len()).step_by(2) {
        let byte = u8::from_str_radix(&clean[i..i+2], 16).ok()?;
        result.push(byte);
    }
    Some(result)
}

pub fn format_hex(byte: u8) -> String {
    format!("{:02X}", byte)
}

pub fn format_ascii(byte: u8) -> char {
    if byte.is_ascii_graphic() || byte == b' ' {
        byte as char
    } else {
        '.'
    }
}
