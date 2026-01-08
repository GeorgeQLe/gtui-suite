use chrono::Utc;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::config::Config;
use crate::models::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    MailList,
    Reading,
    Compose,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Search,
    Compose,
    AccountSelect,
    MailboxSelect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComposeField {
    To,
    Cc,
    Subject,
    Body,
}

pub struct App {
    pub config: Config,
    pub view: View,
    pub input_mode: InputMode,

    // Accounts
    pub accounts: Vec<Account>,
    pub active_account: usize,
    pub active_mailbox: usize,

    // Emails
    pub emails: Vec<Email>,
    pub selected_email: usize,
    pub reading_email: Option<usize>,

    // Compose
    pub compose: Option<ComposeEmail>,
    pub compose_field: ComposeField,

    // UI state
    pub show_sidebar: bool,
    pub search_query: String,
    pub scroll_offset: usize,

    // Status
    pub status_message: Option<String>,
    pub unread_count: u32,
}

impl App {
    pub fn new(config: Config) -> Self {
        let show_sidebar = config.display.show_sidebar;

        Self {
            config,
            view: View::MailList,
            input_mode: InputMode::Normal,
            accounts: Vec::new(),
            active_account: 0,
            active_mailbox: 0,
            emails: Vec::new(),
            selected_email: 0,
            reading_email: None,
            compose: None,
            compose_field: ComposeField::To,
            show_sidebar,
            search_query: String::new(),
            scroll_offset: 0,
            status_message: None,
            unread_count: 0,
        }
    }

    pub async fn refresh(&mut self) {
        // Create demo account
        let mut account = Account::new(
            "Personal",
            "user@example.com",
            "imap.example.com",
            "smtp.example.com",
            "user@example.com",
        );

        account.mailboxes = vec![
            Mailbox::new("Inbox", "INBOX", MailboxType::Inbox),
            Mailbox::new("Sent", "Sent", MailboxType::Sent),
            Mailbox::new("Drafts", "Drafts", MailboxType::Drafts),
            Mailbox::new("Trash", "Trash", MailboxType::Trash),
            Mailbox::new("Archive", "Archive", MailboxType::Archive),
        ];

        account.mailboxes[0].unread = 3;
        account.mailboxes[0].total = 25;

        self.accounts = vec![account];

        // Load demo emails
        self.load_demo_emails();
    }

    fn load_demo_emails(&mut self) {
        self.emails = vec![
            {
                let mut email = Email::new(
                    "Weekly Team Update",
                    Address::with_name("Alice Smith", "alice@company.com"),
                    vec![Address::new("user@example.com")],
                );
                email.date = Utc::now() - chrono::Duration::hours(2);
                email.body_text = Some("Hi team,\n\nHere's the weekly update:\n\n- Project A is on track\n- Need input on Project B\n- Meeting scheduled for Friday\n\nBest,\nAlice".to_string());
                email.flags.seen = false;
                email
            },
            {
                let mut email = Email::new(
                    "Re: Meeting Tomorrow",
                    Address::with_name("Bob Johnson", "bob@company.com"),
                    vec![Address::new("user@example.com")],
                );
                email.date = Utc::now() - chrono::Duration::hours(5);
                email.body_text = Some("Sounds good, I'll be there.\n\n> Can we meet at 2pm?\n> \n> Thanks,\n> User".to_string());
                email.flags.seen = true;
                email.flags.answered = true;
                email
            },
            {
                let mut email = Email::new(
                    "Important: Security Update Required",
                    Address::with_name("IT Department", "it@company.com"),
                    vec![Address::new("user@example.com")],
                );
                email.date = Utc::now() - chrono::Duration::days(1);
                email.body_text = Some("Please update your password before end of week.\n\nRegards,\nIT Team".to_string());
                email.flags.seen = false;
                email.flags.flagged = true;
                email
            },
            {
                let mut email = Email::new(
                    "Vacation Request Approved",
                    Address::with_name("HR System", "hr@company.com"),
                    vec![Address::new("user@example.com")],
                );
                email.date = Utc::now() - chrono::Duration::days(2);
                email.body_text = Some("Your vacation request has been approved.\n\nDates: Dec 23-27".to_string());
                email.flags.seen = true;
                email
            },
            {
                let mut email = Email::new(
                    "Newsletter: December Edition",
                    Address::with_name("Company Newsletter", "newsletter@company.com"),
                    vec![Address::new("user@example.com")],
                );
                email.date = Utc::now() - chrono::Duration::days(3);
                email.body_text = Some("This month's highlights...\n\n- Year in review\n- Upcoming events\n- Employee spotlight".to_string());
                email.flags.seen = false;
                email
            },
        ];

        self.unread_count = self.emails.iter().filter(|e| !e.flags.seen).count() as u32;
    }

    pub async fn handle_key(&mut self, key: KeyEvent) -> bool {
        match self.input_mode {
            InputMode::Normal => self.handle_normal_key(key).await,
            InputMode::Search => self.handle_search_key(key),
            InputMode::Compose => self.handle_compose_key(key).await,
            InputMode::AccountSelect => self.handle_account_select_key(key),
            InputMode::MailboxSelect => self.handle_mailbox_select_key(key),
        }
    }

    async fn handle_normal_key(&mut self, key: KeyEvent) -> bool {
        let is_ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

        match key.code {
            KeyCode::Char('q') if is_ctrl => return true,
            KeyCode::Char('q') => {
                if self.view == View::Reading {
                    self.view = View::MailList;
                    self.reading_email = None;
                } else {
                    return true;
                }
            }

            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected_email < self.emails.len().saturating_sub(1) {
                    self.selected_email += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_email = self.selected_email.saturating_sub(1);
            }
            KeyCode::Char('g') => {
                self.selected_email = 0;
            }
            KeyCode::Char('G') => {
                self.selected_email = self.emails.len().saturating_sub(1);
            }

            KeyCode::Enter | KeyCode::Char('l') => {
                if !self.emails.is_empty() {
                    self.view = View::Reading;
                    self.reading_email = Some(self.selected_email);
                    if let Some(email) = self.emails.get_mut(self.selected_email) {
                        if !email.flags.seen {
                            email.flags.seen = true;
                            self.unread_count = self.unread_count.saturating_sub(1);
                        }
                    }
                }
            }

            KeyCode::Char('c') => {
                self.compose = Some(ComposeEmail::default());
                self.compose_field = ComposeField::To;
                self.input_mode = InputMode::Compose;
                self.view = View::Compose;
            }

            KeyCode::Char('r') => {
                if let Some(email) = self.emails.get(self.selected_email) {
                    self.compose = Some(ComposeEmail::reply(email, false));
                    self.compose_field = ComposeField::Body;
                    self.input_mode = InputMode::Compose;
                    self.view = View::Compose;
                }
            }

            KeyCode::Char('R') => {
                if let Some(email) = self.emails.get(self.selected_email) {
                    self.compose = Some(ComposeEmail::reply(email, true));
                    self.compose_field = ComposeField::Body;
                    self.input_mode = InputMode::Compose;
                    self.view = View::Compose;
                }
            }

            KeyCode::Char('f') => {
                if let Some(email) = self.emails.get(self.selected_email) {
                    self.compose = Some(ComposeEmail::forward(email));
                    self.compose_field = ComposeField::To;
                    self.input_mode = InputMode::Compose;
                    self.view = View::Compose;
                }
            }

            KeyCode::Char('d') => {
                if !self.emails.is_empty() {
                    if let Some(email) = self.emails.get_mut(self.selected_email) {
                        email.flags.deleted = true;
                    }
                    self.status_message = Some("Email moved to trash".to_string());
                }
            }

            KeyCode::Char('s') => {
                if let Some(email) = self.emails.get_mut(self.selected_email) {
                    email.flags.flagged = !email.flags.flagged;
                }
            }

            KeyCode::Char('/') => {
                self.input_mode = InputMode::Search;
                self.search_query.clear();
            }

            KeyCode::Char('a') => {
                self.input_mode = InputMode::AccountSelect;
            }

            KeyCode::Char('m') => {
                self.input_mode = InputMode::MailboxSelect;
            }

            KeyCode::Char('b') => {
                self.show_sidebar = !self.show_sidebar;
            }

            _ => {}
        }

        false
    }

    fn handle_search_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
                self.search_query.clear();
            }
            KeyCode::Enter => {
                self.input_mode = InputMode::Normal;
                // Would filter emails by search_query
            }
            KeyCode::Backspace => {
                self.search_query.pop();
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
            }
            _ => {}
        }
        false
    }

    async fn handle_compose_key(&mut self, key: KeyEvent) -> bool {
        let is_ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
                self.view = View::MailList;
                self.compose = None;
            }
            KeyCode::Tab => {
                self.compose_field = match self.compose_field {
                    ComposeField::To => ComposeField::Cc,
                    ComposeField::Cc => ComposeField::Subject,
                    ComposeField::Subject => ComposeField::Body,
                    ComposeField::Body => ComposeField::To,
                };
            }
            KeyCode::BackTab => {
                self.compose_field = match self.compose_field {
                    ComposeField::To => ComposeField::Body,
                    ComposeField::Cc => ComposeField::To,
                    ComposeField::Subject => ComposeField::Cc,
                    ComposeField::Body => ComposeField::Subject,
                };
            }
            KeyCode::Enter if is_ctrl => {
                // Send email
                self.status_message = Some("Email sent!".to_string());
                self.input_mode = InputMode::Normal;
                self.view = View::MailList;
                self.compose = None;
            }
            KeyCode::Backspace => {
                if let Some(compose) = &mut self.compose {
                    let field = match self.compose_field {
                        ComposeField::To => &mut compose.to,
                        ComposeField::Cc => &mut compose.cc,
                        ComposeField::Subject => &mut compose.subject,
                        ComposeField::Body => &mut compose.body,
                    };
                    field.pop();
                }
            }
            KeyCode::Enter => {
                if self.compose_field == ComposeField::Body {
                    if let Some(compose) = &mut self.compose {
                        compose.body.push('\n');
                    }
                } else {
                    // Move to next field
                    self.compose_field = match self.compose_field {
                        ComposeField::To => ComposeField::Cc,
                        ComposeField::Cc => ComposeField::Subject,
                        ComposeField::Subject => ComposeField::Body,
                        ComposeField::Body => ComposeField::Body,
                    };
                }
            }
            KeyCode::Char(c) => {
                if let Some(compose) = &mut self.compose {
                    let field = match self.compose_field {
                        ComposeField::To => &mut compose.to,
                        ComposeField::Cc => &mut compose.cc,
                        ComposeField::Subject => &mut compose.subject,
                        ComposeField::Body => &mut compose.body,
                    };
                    field.push(c);
                }
            }
            _ => {}
        }

        false
    }

    fn handle_account_select_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.active_account =
                    (self.active_account + 1).min(self.accounts.len().saturating_sub(1));
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.active_account = self.active_account.saturating_sub(1);
            }
            KeyCode::Enter => {
                self.input_mode = InputMode::Normal;
                self.active_mailbox = 0;
            }
            _ => {}
        }
        false
    }

    fn handle_mailbox_select_key(&mut self, key: KeyEvent) -> bool {
        let mailbox_count = self
            .accounts
            .get(self.active_account)
            .map(|a| a.mailboxes.len())
            .unwrap_or(0);

        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.active_mailbox = (self.active_mailbox + 1).min(mailbox_count.saturating_sub(1));
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.active_mailbox = self.active_mailbox.saturating_sub(1);
            }
            KeyCode::Enter => {
                self.input_mode = InputMode::Normal;
                self.selected_email = 0;
            }
            _ => {}
        }
        false
    }

    pub async fn check_new_mail(&mut self) {
        // Would check for new mail via IMAP
        // For demo, this is a no-op
    }

    pub fn current_account(&self) -> Option<&Account> {
        self.accounts.get(self.active_account)
    }

    pub fn current_mailbox(&self) -> Option<&Mailbox> {
        self.current_account()
            .and_then(|a| a.mailboxes.get(self.active_mailbox))
    }

    pub fn current_email(&self) -> Option<&Email> {
        self.reading_email.and_then(|i| self.emails.get(i))
    }

    pub fn status_text(&self) -> String {
        if let Some(msg) = &self.status_message {
            return msg.clone();
        }

        match self.view {
            View::MailList => {
                format!(
                    "{} emails ({} unread) | j/k:navigate  Enter:read  c:compose  r:reply  q:quit",
                    self.emails.len(),
                    self.unread_count
                )
            }
            View::Reading => "q:back  r:reply  R:reply-all  f:forward  d:delete".to_string(),
            View::Compose => "Tab:next field  Ctrl+Enter:send  Esc:cancel".to_string(),
        }
    }
}
