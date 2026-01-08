use serde::{Deserialize, Serialize};

pub type TabId = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TabLayout {
    Single {
        app_name: String,
        app_title: String,
    },
    Split {
        direction: Direction,
        ratio: f32,
        children: Box<(TabLayout, TabLayout)>,
        focused: usize, // 0 or 1
    },
}

impl TabLayout {
    pub fn single(name: &str, title: &str) -> Self {
        TabLayout::Single {
            app_name: name.to_string(),
            app_title: title.to_string(),
        }
    }

    pub fn title(&self) -> &str {
        match self {
            TabLayout::Single { app_title, .. } => app_title,
            TabLayout::Split { children, focused, .. } => {
                if *focused == 0 {
                    children.0.title()
                } else {
                    children.1.title()
                }
            }
        }
    }

    pub fn focused_name(&self) -> &str {
        match self {
            TabLayout::Single { app_name, .. } => app_name,
            TabLayout::Split { children, focused, .. } => {
                if *focused == 0 {
                    children.0.focused_name()
                } else {
                    children.1.focused_name()
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tab {
    pub id: TabId,
    pub layout: TabLayout,
    pub pinned: bool,
}

impl Tab {
    pub fn new(id: TabId, name: &str, title: &str) -> Self {
        Self {
            id,
            layout: TabLayout::single(name, title),
            pinned: false,
        }
    }

    pub fn title(&self) -> &str {
        self.layout.title()
    }

    pub fn split(&mut self, direction: Direction, new_name: &str, new_title: &str) {
        let old_layout = std::mem::replace(
            &mut self.layout,
            TabLayout::single("temp", "temp"),
        );

        self.layout = TabLayout::Split {
            direction,
            ratio: 0.5,
            children: Box::new((old_layout, TabLayout::single(new_name, new_title))),
            focused: 1,
        };
    }

    pub fn focus_pane(&mut self, direction: Direction) {
        if let TabLayout::Split {
            direction: split_dir,
            focused,
            ..
        } = &mut self.layout
        {
            if *split_dir == direction {
                *focused = if *focused == 0 { 1 } else { 0 };
            }
        }
    }

    pub fn resize_split(&mut self, delta: f32) {
        if let TabLayout::Split { ratio, .. } = &mut self.layout {
            *ratio = (*ratio + delta).clamp(0.2, 0.8);
        }
    }

    pub fn close_focused_pane(&mut self) -> bool {
        if let TabLayout::Split { children, focused, .. } = &mut self.layout {
            let remaining = if *focused == 0 {
                children.1.clone()
            } else {
                children.0.clone()
            };
            self.layout = remaining;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_new() {
        let tab = Tab::new(1, "app", "App Title");
        assert_eq!(tab.id, 1);
        assert_eq!(tab.title(), "App Title");
        assert!(!tab.pinned);
    }

    #[test]
    fn test_tab_split() {
        let mut tab = Tab::new(1, "app1", "App 1");
        tab.split(Direction::Horizontal, "app2", "App 2");

        if let TabLayout::Split { direction, focused, .. } = &tab.layout {
            assert_eq!(*direction, Direction::Horizontal);
            assert_eq!(*focused, 1);
        } else {
            panic!("Expected split layout");
        }
    }

    #[test]
    fn test_close_pane() {
        let mut tab = Tab::new(1, "app1", "App 1");
        tab.split(Direction::Horizontal, "app2", "App 2");

        assert!(tab.close_focused_pane());
        assert!(matches!(tab.layout, TabLayout::Single { .. }));
    }
}
