use serde::{Deserialize, Serialize};

pub type ContainerId = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    Horizontal,
    Vertical,
}

impl Direction {
    pub fn toggle(&self) -> Self {
        match self {
            Direction::Horizontal => Direction::Vertical,
            Direction::Vertical => Direction::Horizontal,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Container {
    Split {
        id: ContainerId,
        direction: Direction,
        children: Vec<Container>,
        ratios: Vec<f32>,
        focused: usize,
    },
    Tabbed {
        id: ContainerId,
        children: Vec<Container>,
        active: usize,
    },
    App {
        id: ContainerId,
        name: String,
        title: String,
    },
    Empty {
        id: ContainerId,
    },
}

impl Container {
    pub fn id(&self) -> ContainerId {
        match self {
            Container::Split { id, .. } => *id,
            Container::Tabbed { id, .. } => *id,
            Container::App { id, .. } => *id,
            Container::Empty { id } => *id,
        }
    }

    pub fn new_empty(id: ContainerId) -> Self {
        Container::Empty { id }
    }

    pub fn new_app(id: ContainerId, name: String, title: String) -> Self {
        Container::App { id, name, title }
    }

    pub fn new_split(id: ContainerId, direction: Direction, children: Vec<Container>) -> Self {
        let len = children.len();
        let ratio = 1.0 / len as f32;
        Container::Split {
            id,
            direction,
            children,
            ratios: vec![ratio; len],
            focused: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        matches!(self, Container::Empty { .. })
    }

    pub fn child_count(&self) -> usize {
        match self {
            Container::Split { children, .. } => children.len(),
            Container::Tabbed { children, .. } => children.len(),
            Container::App { .. } => 0,
            Container::Empty { .. } => 0,
        }
    }

    pub fn find_focused_app(&self) -> Option<&Container> {
        match self {
            Container::App { .. } => Some(self),
            Container::Empty { .. } => None,
            Container::Split { children, focused, .. } => {
                children.get(*focused).and_then(|c| c.find_focused_app())
            }
            Container::Tabbed { children, active, .. } => {
                children.get(*active).and_then(|c| c.find_focused_app())
            }
        }
    }

    pub fn find_focused_app_mut(&mut self) -> Option<&mut Container> {
        match self {
            Container::App { .. } => Some(self),
            Container::Empty { .. } => None,
            Container::Split { children, focused, .. } => {
                children.get_mut(*focused).and_then(|c| c.find_focused_app_mut())
            }
            Container::Tabbed { children, active, .. } => {
                children.get_mut(*active).and_then(|c| c.find_focused_app_mut())
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: u32,
    pub name: String,
    pub root: Container,
}

impl Workspace {
    pub fn new(id: u32, name: String) -> Self {
        Self {
            id,
            name,
            root: Container::Empty { id: 0 },
        }
    }

    pub fn is_empty(&self) -> bool {
        self.root.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction_toggle() {
        assert_eq!(Direction::Horizontal.toggle(), Direction::Vertical);
        assert_eq!(Direction::Vertical.toggle(), Direction::Horizontal);
    }

    #[test]
    fn test_container_id() {
        let empty = Container::Empty { id: 42 };
        assert_eq!(empty.id(), 42);

        let app = Container::new_app(1, "test".to_string(), "Test App".to_string());
        assert_eq!(app.id(), 1);
    }

    #[test]
    fn test_workspace_empty() {
        let ws = Workspace::new(1, "main".to_string());
        assert!(ws.is_empty());
    }
}
