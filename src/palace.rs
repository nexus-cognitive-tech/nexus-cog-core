//! Memory-palace types.
//!
//! The memory palace is a room-based memory model where each room represents a category
//! of knowledge, items inside rooms are individual memories, and connections between
//! rooms represent semantic links.

use serde::{Deserialize, Serialize};

/// Type of a memory-palace room.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoomType {
    /// A conceptual room (e.g. "Concurrency").
    Concept,
    /// A pattern room (e.g. "Builder pattern examples").
    Pattern,
    /// A decision room (architectural decisions).
    Decision,
    /// A bug room (recurring bugs and their fixes).
    Bug,
    /// A learning room (general learnings).
    Learning,
    /// A tool room (knowledge about specific tools).
    Tool,
    /// A user room (user preferences).
    User,
    /// A project room (project-specific context).
    Project,
}

impl RoomType {
    /// Stable identifier.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Concept => "concept",
            Self::Pattern => "pattern",
            Self::Decision => "decision",
            Self::Bug => "bug",
            Self::Learning => "learning",
            Self::Tool => "tool",
            Self::User => "user",
            Self::Project => "project",
        }
    }
}

/// An item inside a room.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryItem {
    /// Stable identifier.
    pub id: String,
    /// Key (lookup).
    pub key: String,
    /// Value.
    pub value: String,
    /// Confidence in `[0.0, 1.0]`.
    pub confidence: f32,
    /// Number of times this item has been recalled.
    pub access_count: u32,
    /// Unix timestamp (seconds) of last access.
    pub last_accessed: i64,
    /// Tags.
    #[serde(default)]
    pub tags: Vec<String>,
}

impl MemoryItem {
    /// Construct a new memory item.
    ///
    /// `last_accessed` is initialised to the current Unix timestamp so the item
    /// is **not** immediately eligible for TTL pruning on its first decay pass.
    #[must_use]
    pub fn new(key: impl Into<String>, value: impl Into<String>, confidence: f32) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            key: key.into(),
            value: value.into(),
            confidence: confidence.clamp(0.0, 1.0),
            access_count: 0,
            last_accessed: chrono::Utc::now().timestamp(),
            tags: Vec::new(),
        }
    }
}

/// A room in the memory palace.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Room {
    /// Stable identifier.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Room type.
    pub room_type: RoomType,
    /// Items in this room.
    pub items: Vec<MemoryItem>,
    /// Importance in `[0.0, 1.0]`.
    pub importance: f32,
    /// Description.
    #[serde(default)]
    pub description: String,
    /// Tags.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Unix timestamp (seconds) when the room was created.
    pub created_at: i64,
    /// Unix timestamp (seconds) of last update.
    pub updated_at: i64,
}

/// A connection between two rooms.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Connection {
    /// Source room ID.
    pub from: String,
    /// Target room ID.
    pub to: String,
    /// Strength in `[0.0, 1.0]`.
    pub strength: f32,
    /// Relation description (e.g. `"uses"`, `"derived_from"`).
    pub relation: String,
    /// Optional notes.
    #[serde(default)]
    pub notes: String,
}

/// Summary of the memory palace.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PalaceSummary {
    /// Total rooms.
    pub total_rooms: usize,
    /// Total items across all rooms.
    pub total_items: usize,
    /// Total connections.
    pub total_connections: usize,
    /// Average room importance.
    pub avg_importance: f32,
    /// Total recall events.
    pub total_recalls: u64,
}

/// The full memory palace.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryPalace {
    /// All rooms.
    pub rooms: Vec<Room>,
    /// All connections.
    pub connections: Vec<Connection>,
    /// Stable identifier.
    pub id: String,
    /// Timestamp.
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Optional name for the palace.
    #[serde(default)]
    pub name: String,
}

impl MemoryPalace {
    /// Find a room by ID.
    #[must_use]
    pub fn room(&self, id: &str) -> Option<&Room> {
        self.rooms.iter().find(|r| r.id == id)
    }

    /// Find rooms of a given type.
    pub fn rooms_of_type(&self, room_type: RoomType) -> impl Iterator<Item = &Room> {
        self.rooms.iter().filter(move |r| r.room_type == room_type)
    }

    /// Find items across all rooms matching a key.
    #[must_use]
    pub fn find_items(&self, key: &str) -> Vec<&MemoryItem> {
        self.rooms
            .iter()
            .flat_map(|r| r.items.iter())
            .filter(|i| i.key == key)
            .collect()
    }

    /// Rooms connected to the given room.
    #[must_use]
    pub fn connected_rooms(&self, room_id: &str) -> Vec<&Room> {
        let connected_ids: Vec<&str> = self
            .connections
            .iter()
            .filter(|c| c.from == room_id || c.to == room_id)
            .map(|c| {
                if c.from == room_id {
                    c.to.as_str()
                } else {
                    c.from.as_str()
                }
            })
            .collect();
        self.rooms
            .iter()
            .filter(|r| connected_ids.contains(&r.id.as_str()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_palace() -> MemoryPalace {
        MemoryPalace {
            rooms: vec![
                Room {
                    id: "r1".into(),
                    name: "Concurrency".into(),
                    room_type: RoomType::Concept,
                    items: vec![MemoryItem::new("async", "use tokio", 0.9)],
                    importance: 0.8,
                    description: String::new(),
                    tags: vec![],
                    created_at: 0,
                    updated_at: 0,
                },
                Room {
                    id: "r2".into(),
                    name: "Bugs".into(),
                    room_type: RoomType::Bug,
                    items: vec![],
                    importance: 0.5,
                    description: String::new(),
                    tags: vec![],
                    created_at: 0,
                    updated_at: 0,
                },
            ],
            connections: vec![Connection {
                from: "r1".into(),
                to: "r2".into(),
                strength: 0.6,
                relation: "relates_to".into(),
                notes: String::new(),
            }],
            id: "p1".into(),
            timestamp: chrono::Utc::now(),
            name: "test".into(),
        }
    }

    #[test]
    fn find_items_returns_matches() {
        let p = make_palace();
        let items = p.find_items("async");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].value, "use tokio");
    }

    #[test]
    fn connected_rooms_returns_neighbors() {
        let p = make_palace();
        let neighbors = p.connected_rooms("r1");
        assert_eq!(neighbors.len(), 1);
        assert_eq!(neighbors[0].id, "r2");
    }

    #[test]
    fn rooms_of_type_filters() {
        let p = make_palace();
        let bugs: Vec<_> = p.rooms_of_type(RoomType::Bug).collect();
        assert_eq!(bugs.len(), 1);
    }
}
