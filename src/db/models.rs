use serde::{Deserialize, Serialize};

// ─── Workspace ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_color")]
    pub color: String,
    #[serde(default = "default_icon")]
    pub icon: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

fn default_color() -> String { "#6366f1".into() }
fn default_icon() -> String { "📚".into() }

// ─── Topic ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Topic {
    pub id: String,
    pub workspace_id: Option<String>,
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_status")]
    pub status: String,
    pub explored_at: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

fn default_status() -> String { "pending".into() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicWithWorkspace {
    #[serde(flatten)]
    pub topic: Topic,
    pub workspace_name: Option<String>,
    pub workspace_color: Option<String>,
    pub workspace_icon: Option<String>,
}

// ─── Concept ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Concept {
    pub id: String,
    pub topic_id: String,
    pub name: String,
    #[serde(default = "default_concept_type")]
    pub r#type: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_importance")]
    pub importance: i32,
    #[serde(default)]
    pub details: String,
    #[serde(default = "default_json_array")]
    pub code_examples: String,
    #[serde(default = "default_json_array")]
    pub external_resources: String,
    pub parent_concept_id: Option<String>,
    #[serde(default)]
    pub depth: i32,
    #[serde(default)]
    pub explored: bool,
    pub created_at: Option<String>,
}

fn default_concept_type() -> String { "concept".into() }
fn default_importance() -> i32 { 5 }
fn default_json_array() -> String { "[]".into() }

// ─── Relationship ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub id: String,
    pub topic_id: String,
    pub source_concept_id: String,
    pub target_concept_id: String,
    #[serde(default = "default_rel_type")]
    pub relationship_type: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_importance")]
    pub strength: i32,
    pub created_at: Option<String>,
}

fn default_rel_type() -> String { "relates_to".into() }

// ─── KnowledgeCard ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeCard {
    pub id: String,
    pub concept_id: Option<String>,
    pub topic_id: String,
    #[serde(default = "default_card_type")]
    pub card_type: String,
    pub front: String,
    pub back: String,
    #[serde(default = "default_difficulty")]
    pub difficulty: i32,
    #[serde(default = "default_json_array")]
    pub tags: String,
    pub created_at: Option<String>,
}

fn default_card_type() -> String { "fact".into() }
fn default_difficulty() -> i32 { 1 }

// ─── TimelineEvent ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEvent {
    pub id: String,
    pub topic_id: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub date_label: String,
    #[serde(default)]
    pub period: String,
    #[serde(default)]
    pub order_index: i32,
    #[serde(default = "default_event_importance")]
    pub importance: String,
    #[serde(default = "default_category")]
    pub category: String,
    pub created_at: Option<String>,
}

fn default_event_importance() -> String { "medium".into() }
fn default_category() -> String { "general".into() }

// ─── Note ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub topic_id: Option<String>,
    pub workspace_id: Option<String>,
    #[serde(default = "default_note_title")]
    pub title: String,
    #[serde(default)]
    pub content: String,
    #[serde(default = "default_json_array")]
    pub tags: String,
    #[serde(default)]
    pub is_pinned: bool,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

fn default_note_title() -> String { "Untitled Note".into() }

// ─── Flashcard ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Flashcard {
    pub id: String,
    pub topic_id: String,
    pub concept_id: Option<String>,
    pub question: String,
    pub answer: String,
    #[serde(default = "default_difficulty")]
    pub difficulty: i32,
    #[serde(default)]
    pub times_reviewed: i32,
    #[serde(default)]
    pub times_correct: i32,
    pub last_reviewed: Option<String>,
    pub next_review: Option<String>,
    pub created_at: Option<String>,
}

// ─── LearningPath ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningPath {
    pub id: String,
    pub topic_id: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_json_array")]
    pub steps: String,
    #[serde(default = "default_lp_difficulty")]
    pub difficulty: String,
    #[serde(default)]
    pub estimated_time: String,
    pub created_at: Option<String>,
}

fn default_lp_difficulty() -> String { "beginner".into() }

// ─── Conversation ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub topic_id: String,
    #[serde(default = "default_convo_title")]
    pub title: String,
    #[serde(default = "default_json_array")]
    pub messages: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

fn default_convo_title() -> String { "New Conversation".into() }

// ─── Bookmark ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: String,
    pub topic_id: String,
    pub concept_id: Option<String>,
    #[serde(default)]
    pub note: String,
    pub created_at: Option<String>,
}

// ─── AppSettings ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub id: String,
    #[serde(default = "default_provider")]
    pub ai_provider: String,
    #[serde(default = "default_model")]
    pub ai_model: String,
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub api_endpoint: String,
    #[serde(default = "default_theme")]
    pub theme: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

fn default_provider() -> String { "ollama".into() }
fn default_model() -> String { "llama3".into() }
fn default_theme() -> String { "dark".into() }

// ─── AI Types ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIExplorationResult {
    pub concepts: Vec<ConceptInput>,
    pub relationships: Vec<RelationshipInput>,
    #[serde(default)]
    pub timeline: Vec<TimelineEventInput>,
    #[serde(default)]
    pub flashcards: Vec<FlashcardInput>,
    #[serde(default)]
    pub learning_path: Option<LearningPathInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptInput {
    pub name: String,
    #[serde(default = "default_concept_type")]
    pub r#type: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_importance")]
    pub importance: i32,
    #[serde(default)]
    pub details: String,
    #[serde(default)]
    pub code_examples: Vec<serde_json::Value>,
    #[serde(default)]
    pub external_resources: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipInput {
    pub source: String,
    pub target: String,
    #[serde(default = "default_rel_type")]
    pub relationship_type: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_importance")]
    pub strength: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEventInput {
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub date_label: String,
    #[serde(default)]
    pub period: String,
    #[serde(default)]
    pub order_index: i32,
    #[serde(default = "default_event_importance")]
    pub importance: String,
    #[serde(default = "default_category")]
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashcardInput {
    pub question: String,
    pub answer: String,
    #[serde(default = "default_difficulty")]
    pub difficulty: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningPathInput {
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub steps: Vec<LearningStepInput>,
    #[serde(default = "default_lp_difficulty")]
    pub difficulty: String,
    #[serde(default)]
    pub estimated_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningStepInput {
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub resources: Vec<String>,
    #[serde(default)]
    pub order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizQuestion {
    pub question: String,
    pub options: Vec<String>,
    pub correct_answer: usize,
    #[serde(default)]
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}
