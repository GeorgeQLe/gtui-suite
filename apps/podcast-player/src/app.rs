use chrono::{DateTime, Utc};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone)]
pub struct Podcast {
    pub title: String,
    pub author: String,
    pub episodes: Vec<Episode>,
}

#[derive(Debug, Clone)]
pub struct Episode {
    pub title: String,
    pub published: DateTime<Utc>,
    pub duration: u32, // seconds
    pub played: bool,
    pub progress: u32, // seconds
}

impl Episode {
    pub fn duration_formatted(&self) -> String {
        format_duration(self.duration)
    }

    pub fn progress_formatted(&self) -> String {
        format_duration(self.progress)
    }

    pub fn progress_percent(&self) -> f64 {
        if self.duration == 0 {
            0.0
        } else {
            (self.progress as f64 / self.duration as f64) * 100.0
        }
    }
}

fn format_duration(seconds: u32) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if hours > 0 {
        format!("{}:{:02}:{:02}", hours, minutes, secs)
    } else {
        format!("{}:{:02}", minutes, secs)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Podcasts,
    Episodes,
    Player,
}

pub struct App {
    pub podcasts: Vec<Podcast>,
    pub selected_podcast: usize,
    pub selected_episode: usize,
    pub view: View,
    pub is_playing: bool,
    pub current_podcast: Option<usize>,
    pub current_episode: Option<usize>,
    pub volume: u8,
    pub playback_speed: f32,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            podcasts: Vec::new(),
            selected_podcast: 0,
            selected_episode: 0,
            view: View::Podcasts,
            is_playing: false,
            current_podcast: None,
            current_episode: None,
            volume: 80,
            playback_speed: 1.0,
            status_message: None,
        }
    }

    pub fn load_podcasts(&mut self) {
        self.podcasts = create_demo_podcasts();
    }

    pub fn update_playback(&mut self) {
        if self.is_playing {
            if let (Some(pi), Some(ei)) = (self.current_podcast, self.current_episode) {
                if let Some(podcast) = self.podcasts.get_mut(pi) {
                    if let Some(episode) = podcast.episodes.get_mut(ei) {
                        if episode.progress < episode.duration {
                            episode.progress += 1;
                        } else {
                            self.is_playing = false;
                            episode.played = true;
                        }
                    }
                }
            }
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return true;
        }

        match self.view {
            View::Podcasts => self.handle_podcasts_key(key),
            View::Episodes => self.handle_episodes_key(key),
            View::Player => self.handle_player_key(key),
        }
    }

    fn handle_podcasts_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected_podcast < self.podcasts.len().saturating_sub(1) {
                    self.selected_podcast += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_podcast = self.selected_podcast.saturating_sub(1);
            }
            KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => {
                self.view = View::Episodes;
                self.selected_episode = 0;
            }
            KeyCode::Char(' ') => {
                self.toggle_playback();
            }
            _ => {}
        }
        false
    }

    fn handle_episodes_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Esc | KeyCode::Char('h') | KeyCode::Left => {
                self.view = View::Podcasts;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if let Some(podcast) = self.podcasts.get(self.selected_podcast) {
                    if self.selected_episode < podcast.episodes.len().saturating_sub(1) {
                        self.selected_episode += 1;
                    }
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_episode = self.selected_episode.saturating_sub(1);
            }
            KeyCode::Enter => {
                self.play_selected_episode();
            }
            KeyCode::Char(' ') => {
                self.toggle_playback();
            }
            KeyCode::Char('p') => {
                self.view = View::Player;
            }
            _ => {}
        }
        false
    }

    fn handle_player_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.view = View::Episodes;
            }
            KeyCode::Char(' ') => {
                self.toggle_playback();
            }
            KeyCode::Char('+') | KeyCode::Char('=') => {
                self.volume = (self.volume + 5).min(100);
            }
            KeyCode::Char('-') => {
                self.volume = self.volume.saturating_sub(5);
            }
            KeyCode::Char('[') => {
                self.playback_speed = (self.playback_speed - 0.25).max(0.5);
            }
            KeyCode::Char(']') => {
                self.playback_speed = (self.playback_speed + 0.25).min(3.0);
            }
            KeyCode::Left => {
                self.seek(-10);
            }
            KeyCode::Right => {
                self.seek(10);
            }
            _ => {}
        }
        false
    }

    fn play_selected_episode(&mut self) {
        self.current_podcast = Some(self.selected_podcast);
        self.current_episode = Some(self.selected_episode);
        self.is_playing = true;
        self.view = View::Player;

        if let Some(episode) = self.current_episode_mut() {
            if episode.progress >= episode.duration {
                episode.progress = 0;
            }
        }
    }

    fn toggle_playback(&mut self) {
        if self.current_podcast.is_some() && self.current_episode.is_some() {
            self.is_playing = !self.is_playing;
            self.status_message = Some(if self.is_playing {
                "Playing".to_string()
            } else {
                "Paused".to_string()
            });
        }
    }

    fn seek(&mut self, seconds: i32) {
        if let Some(episode) = self.current_episode_mut() {
            let new_progress = episode.progress as i32 + seconds;
            episode.progress = new_progress.max(0).min(episode.duration as i32) as u32;
        }
    }

    pub fn current_episode_ref(&self) -> Option<&Episode> {
        let pi = self.current_podcast?;
        let ei = self.current_episode?;
        self.podcasts.get(pi)?.episodes.get(ei)
    }

    fn current_episode_mut(&mut self) -> Option<&mut Episode> {
        let pi = self.current_podcast?;
        let ei = self.current_episode?;
        self.podcasts.get_mut(pi)?.episodes.get_mut(ei)
    }

    pub fn current_podcast_ref(&self) -> Option<&Podcast> {
        self.podcasts.get(self.current_podcast?)
    }

    pub fn status_text(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }

        match self.view {
            View::Podcasts => "Enter:episodes Space:play/pause".to_string(),
            View::Episodes => "Enter:play Space:play/pause p:player Esc:back".to_string(),
            View::Player => format!(
                "Space:play/pause ←→:seek +/-:volume [{:.1}x] Vol:{}%",
                self.playback_speed, self.volume
            ),
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn create_demo_podcasts() -> Vec<Podcast> {
    vec![
        Podcast {
            title: "Rust Radio".to_string(),
            author: "The Rust Team".to_string(),
            episodes: vec![
                Episode {
                    title: "Episode 42: Async Rust Deep Dive".to_string(),
                    published: Utc::now(),
                    duration: 3600,
                    played: false,
                    progress: 0,
                },
                Episode {
                    title: "Episode 41: Building CLI Tools".to_string(),
                    published: Utc::now(),
                    duration: 2700,
                    played: true,
                    progress: 2700,
                },
                Episode {
                    title: "Episode 40: Macro Magic".to_string(),
                    published: Utc::now(),
                    duration: 3200,
                    played: false,
                    progress: 1500,
                },
            ],
        },
        Podcast {
            title: "Terminal Talk".to_string(),
            author: "CLI Enthusiasts".to_string(),
            episodes: vec![
                Episode {
                    title: "TUI Frameworks Compared".to_string(),
                    published: Utc::now(),
                    duration: 2400,
                    played: false,
                    progress: 0,
                },
                Episode {
                    title: "The History of the Terminal".to_string(),
                    published: Utc::now(),
                    duration: 1800,
                    played: true,
                    progress: 1800,
                },
            ],
        },
        Podcast {
            title: "Dev Diaries".to_string(),
            author: "Various".to_string(),
            episodes: vec![
                Episode {
                    title: "Open Source Sustainability".to_string(),
                    published: Utc::now(),
                    duration: 4500,
                    played: false,
                    progress: 0,
                },
                Episode {
                    title: "Remote Work Best Practices".to_string(),
                    published: Utc::now(),
                    duration: 3000,
                    played: false,
                    progress: 0,
                },
                Episode {
                    title: "Code Review Culture".to_string(),
                    published: Utc::now(),
                    duration: 2100,
                    played: true,
                    progress: 2100,
                },
            ],
        },
    ]
}
