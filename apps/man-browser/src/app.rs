use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

#[derive(Debug, Clone)]
pub struct ManPage {
    pub name: String,
    pub section: u8,
    pub description: String,
    pub content: String,
}

impl ManPage {
    pub fn section_name(&self) -> &'static str {
        match self.section {
            1 => "User Commands",
            2 => "System Calls",
            3 => "Library Functions",
            4 => "Special Files",
            5 => "File Formats",
            6 => "Games",
            7 => "Miscellaneous",
            8 => "System Admin",
            9 => "Kernel Routines",
            _ => "Unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    List,
    Reader,
    Search,
}

pub struct App {
    pub pages: Vec<ManPage>,
    pub filtered_pages: Vec<usize>,
    pub selected: usize,
    pub view: View,
    pub search_query: String,
    pub scroll_offset: usize,
    pub section_filter: Option<u8>,
    pub status_message: Option<String>,
    matcher: SkimMatcherV2,
}

impl App {
    pub fn new() -> Self {
        Self {
            pages: Vec::new(),
            filtered_pages: Vec::new(),
            selected: 0,
            view: View::List,
            search_query: String::new(),
            scroll_offset: 0,
            section_filter: None,
            status_message: None,
            matcher: SkimMatcherV2::default(),
        }
    }

    pub fn load_pages(&mut self) {
        self.pages = create_demo_pages();
        self.update_filtered();
    }

    fn update_filtered(&mut self) {
        self.filtered_pages = self
            .pages
            .iter()
            .enumerate()
            .filter(|(_, page)| {
                let section_ok = self
                    .section_filter
                    .map(|s| page.section == s)
                    .unwrap_or(true);

                let search_ok = self.search_query.is_empty()
                    || self
                        .matcher
                        .fuzzy_match(&page.name, &self.search_query)
                        .is_some()
                    || self
                        .matcher
                        .fuzzy_match(&page.description, &self.search_query)
                        .is_some();

                section_ok && search_ok
            })
            .map(|(i, _)| i)
            .collect();

        // Sort by match score if searching
        if !self.search_query.is_empty() {
            let query = self.search_query.clone();
            let matcher = &self.matcher;
            let pages = &self.pages;

            self.filtered_pages.sort_by(|&a, &b| {
                let score_a = matcher
                    .fuzzy_match(&pages[a].name, &query)
                    .unwrap_or(0);
                let score_b = matcher
                    .fuzzy_match(&pages[b].name, &query)
                    .unwrap_or(0);
                score_b.cmp(&score_a)
            });
        }

        self.selected = 0;
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return true;
        }

        match self.view {
            View::List => self.handle_list_key(key),
            View::Reader => self.handle_reader_key(key),
            View::Search => self.handle_search_key(key),
        }
    }

    fn handle_list_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected < self.filtered_pages.len().saturating_sub(1) {
                    self.selected += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
            }
            KeyCode::Enter => {
                if !self.filtered_pages.is_empty() {
                    self.view = View::Reader;
                    self.scroll_offset = 0;
                }
            }
            KeyCode::Char('/') => {
                self.view = View::Search;
            }
            KeyCode::Char(c) if c.is_ascii_digit() && c != '0' => {
                let section = c.to_digit(10).unwrap() as u8;
                if self.section_filter == Some(section) {
                    self.section_filter = None;
                } else {
                    self.section_filter = Some(section);
                }
                self.update_filtered();
            }
            KeyCode::Char('0') => {
                self.section_filter = None;
                self.update_filtered();
            }
            KeyCode::Char('g') => {
                self.selected = 0;
            }
            KeyCode::Char('G') => {
                self.selected = self.filtered_pages.len().saturating_sub(1);
            }
            _ => {}
        }
        false
    }

    fn handle_reader_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.view = View::List;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.scroll_offset += 1;
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
            }
            KeyCode::Char('d') => {
                self.scroll_offset += 10;
            }
            KeyCode::Char('u') => {
                self.scroll_offset = self.scroll_offset.saturating_sub(10);
            }
            KeyCode::Char('g') => {
                self.scroll_offset = 0;
            }
            KeyCode::Char('G') => {
                if let Some(page) = self.selected_page() {
                    self.scroll_offset = page.content.lines().count().saturating_sub(10);
                }
            }
            KeyCode::Char('/') => {
                self.view = View::Search;
            }
            _ => {}
        }
        false
    }

    fn handle_search_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.view = View::List;
            }
            KeyCode::Enter => {
                self.update_filtered();
                self.view = View::List;
            }
            KeyCode::Backspace => {
                self.search_query.pop();
                self.update_filtered();
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
                self.update_filtered();
            }
            _ => {}
        }
        false
    }

    pub fn selected_page(&self) -> Option<&ManPage> {
        self.filtered_pages
            .get(self.selected)
            .and_then(|&idx| self.pages.get(idx))
    }

    pub fn visible_pages(&self) -> Vec<&ManPage> {
        self.filtered_pages
            .iter()
            .filter_map(|&idx| self.pages.get(idx))
            .collect()
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }

        match self.view {
            View::List => {
                let section_info = self
                    .section_filter
                    .map(|s| format!("Section {} ", s))
                    .unwrap_or_default();
                format!(
                    "{}{} pages | Enter:read /:search 1-9:filter section 0:all",
                    section_info,
                    self.filtered_pages.len()
                )
            }
            View::Reader => "j/k:scroll d/u:page g/G:top/bottom /:search Esc:back".to_string(),
            View::Search => format!("Search: {}_", self.search_query),
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn create_demo_pages() -> Vec<ManPage> {
    vec![
        ManPage {
            name: "ls".to_string(),
            section: 1,
            description: "list directory contents".to_string(),
            content: r#"NAME
       ls - list directory contents

SYNOPSIS
       ls [OPTION]... [FILE]...

DESCRIPTION
       List information about the FILEs (the current directory by default).
       Sort entries alphabetically if none of -cftuvSUX nor --sort is specified.

       Mandatory arguments to long options are mandatory for short options too.

       -a, --all
              do not ignore entries starting with .

       -l     use a long listing format

       -h, --human-readable
              with -l and -s, print sizes like 1K 234M 2G etc.

       -r, --reverse
              reverse order while sorting

       -t     sort by time, newest first

EXAMPLES
       ls -la
              List all files in long format

       ls -lh
              List files with human-readable sizes"#
                .to_string(),
        },
        ManPage {
            name: "grep".to_string(),
            section: 1,
            description: "print lines that match patterns".to_string(),
            content: r#"NAME
       grep - print lines that match patterns

SYNOPSIS
       grep [OPTION...] PATTERNS [FILE...]

DESCRIPTION
       grep searches for PATTERNS in each FILE.

       -i, --ignore-case
              Ignore case distinctions in patterns and input data.

       -v, --invert-match
              Invert the sense of matching.

       -r, --recursive
              Read all files under each directory, recursively.

       -n, --line-number
              Prefix each line of output with the line number.

       -E, --extended-regexp
              Interpret PATTERNS as extended regular expressions.

EXAMPLES
       grep -r "pattern" .
              Search recursively in current directory"#
                .to_string(),
        },
        ManPage {
            name: "find".to_string(),
            section: 1,
            description: "search for files in a directory hierarchy".to_string(),
            content: r#"NAME
       find - search for files in a directory hierarchy

SYNOPSIS
       find [path...] [expression]

DESCRIPTION
       GNU find searches the directory tree rooted at each given file name.

       -name pattern
              Base of file name matches shell pattern.

       -type c
              File is of type c: f (regular file), d (directory), l (symlink)

       -mtime n
              File's data was last modified n*24 hours ago.

       -exec command ;
              Execute command.

EXAMPLES
       find . -name "*.txt"
              Find all .txt files

       find . -type d -name "test"
              Find directories named test"#
                .to_string(),
        },
        ManPage {
            name: "chmod".to_string(),
            section: 1,
            description: "change file mode bits".to_string(),
            content: r#"NAME
       chmod - change file mode bits

SYNOPSIS
       chmod [OPTION]... MODE[,MODE]... FILE...

DESCRIPTION
       chmod changes the file mode bits of each given file.

       -R, --recursive
              change files and directories recursively

EXAMPLES
       chmod 755 script.sh
              Make script executable

       chmod -R 644 *.txt
              Change permissions recursively"#
                .to_string(),
        },
        ManPage {
            name: "printf".to_string(),
            section: 3,
            description: "formatted output conversion".to_string(),
            content: r#"NAME
       printf - formatted output conversion

SYNOPSIS
       #include <stdio.h>

       int printf(const char *format, ...);

DESCRIPTION
       The printf() family of functions produces output according to a format.

       Format specifiers:
       %d, %i     - signed decimal integer
       %s         - string
       %f         - decimal floating point
       %x         - unsigned hexadecimal integer

RETURN VALUE
       Upon successful return, these functions return the number of characters
       printed."#
                .to_string(),
        },
        ManPage {
            name: "malloc".to_string(),
            section: 3,
            description: "allocate and free dynamic memory".to_string(),
            content: r#"NAME
       malloc, free - allocate and free dynamic memory

SYNOPSIS
       #include <stdlib.h>

       void *malloc(size_t size);
       void free(void *ptr);

DESCRIPTION
       The malloc() function allocates size bytes and returns a pointer.

RETURN VALUE
       The malloc() function returns a pointer to the allocated memory.
       On error, returns NULL."#
                .to_string(),
        },
        ManPage {
            name: "fstab".to_string(),
            section: 5,
            description: "static information about the filesystems".to_string(),
            content: r#"NAME
       fstab - static information about the filesystems

DESCRIPTION
       The file fstab contains descriptive information about the filesystems.

       Each filesystem is described on a separate line. Fields are separated
       by tabs or spaces.

       The fields are:
       1. fs_spec     - block device or remote filesystem
       2. fs_file     - mount point
       3. fs_vfstype  - filesystem type
       4. fs_mntops   - mount options
       5. fs_freq     - dump frequency
       6. fs_passno   - fsck pass number

FILES
       /etc/fstab"#
                .to_string(),
        },
        ManPage {
            name: "systemd".to_string(),
            section: 8,
            description: "systemd system and service manager".to_string(),
            content: r#"NAME
       systemd - systemd system and service manager

SYNOPSIS
       systemctl [OPTIONS...] COMMAND [NAME...]

DESCRIPTION
       systemd is a system and service manager for Linux operating systems.

COMMANDS
       start NAME...
              Start (activate) one or more units.

       stop NAME...
              Stop (deactivate) one or more units.

       restart NAME...
              Stop and then start one or more units.

       status [NAME...]
              Show runtime status information about units.

       enable NAME...
              Enable one or more units.

       disable NAME...
              Disable one or more units."#
                .to_string(),
        },
    ]
}
