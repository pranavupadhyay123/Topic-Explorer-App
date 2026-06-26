use rusqlite::Connection;

/// Create all database tables if they don't exist
pub fn create_tables(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS workspaces (
            id TEXT PRIMARY KEY NOT NULL,
            name TEXT NOT NULL,
            description TEXT DEFAULT '',
            color TEXT DEFAULT '#6366f1',
            icon TEXT DEFAULT '📚',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS topics (
            id TEXT PRIMARY KEY NOT NULL,
            workspace_id TEXT,
            title TEXT NOT NULL,
            description TEXT DEFAULT '',
            status TEXT DEFAULT 'pending',
            explored_at TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE SET NULL
        );

        CREATE TABLE IF NOT EXISTS concepts (
            id TEXT PRIMARY KEY NOT NULL,
            topic_id TEXT NOT NULL,
            name TEXT NOT NULL,
            type TEXT DEFAULT 'concept',
            description TEXT DEFAULT '',
            importance INTEGER DEFAULT 5,
            details TEXT DEFAULT '',
            code_examples TEXT DEFAULT '[]',
            external_resources TEXT DEFAULT '[]',
            parent_concept_id TEXT,
            depth INTEGER DEFAULT 0,
            explored INTEGER DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (topic_id) REFERENCES topics(id) ON DELETE CASCADE,
            FOREIGN KEY (parent_concept_id) REFERENCES concepts(id) ON DELETE SET NULL
        );

        CREATE TABLE IF NOT EXISTS relationships (
            id TEXT PRIMARY KEY NOT NULL,
            topic_id TEXT NOT NULL,
            source_concept_id TEXT NOT NULL,
            target_concept_id TEXT NOT NULL,
            relationship_type TEXT DEFAULT 'relates_to',
            description TEXT DEFAULT '',
            strength INTEGER DEFAULT 5,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (topic_id) REFERENCES topics(id) ON DELETE CASCADE,
            FOREIGN KEY (source_concept_id) REFERENCES concepts(id) ON DELETE CASCADE,
            FOREIGN KEY (target_concept_id) REFERENCES concepts(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS knowledge_cards (
            id TEXT PRIMARY KEY NOT NULL,
            concept_id TEXT,
            topic_id TEXT NOT NULL,
            card_type TEXT DEFAULT 'fact',
            front TEXT NOT NULL,
            back TEXT NOT NULL,
            difficulty INTEGER DEFAULT 1,
            tags TEXT DEFAULT '[]',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (concept_id) REFERENCES concepts(id) ON DELETE CASCADE,
            FOREIGN KEY (topic_id) REFERENCES topics(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS timeline_events (
            id TEXT PRIMARY KEY NOT NULL,
            topic_id TEXT NOT NULL,
            title TEXT NOT NULL,
            description TEXT DEFAULT '',
            date_label TEXT DEFAULT '',
            period TEXT DEFAULT '',
            order_index INTEGER DEFAULT 0,
            importance TEXT DEFAULT 'medium',
            category TEXT DEFAULT 'general',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (topic_id) REFERENCES topics(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS notes (
            id TEXT PRIMARY KEY NOT NULL,
            topic_id TEXT,
            workspace_id TEXT,
            title TEXT NOT NULL DEFAULT 'Untitled Note',
            content TEXT DEFAULT '',
            tags TEXT DEFAULT '[]',
            is_pinned INTEGER DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (topic_id) REFERENCES topics(id) ON DELETE SET NULL,
            FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE SET NULL
        );

        CREATE TABLE IF NOT EXISTS flashcards (
            id TEXT PRIMARY KEY NOT NULL,
            topic_id TEXT NOT NULL,
            concept_id TEXT,
            question TEXT NOT NULL,
            answer TEXT NOT NULL,
            difficulty INTEGER DEFAULT 1,
            times_reviewed INTEGER DEFAULT 0,
            times_correct INTEGER DEFAULT 0,
            last_reviewed TEXT,
            next_review TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (topic_id) REFERENCES topics(id) ON DELETE CASCADE,
            FOREIGN KEY (concept_id) REFERENCES concepts(id) ON DELETE SET NULL
        );

        CREATE TABLE IF NOT EXISTS learning_paths (
            id TEXT PRIMARY KEY NOT NULL,
            topic_id TEXT NOT NULL,
            title TEXT NOT NULL,
            description TEXT DEFAULT '',
            steps TEXT DEFAULT '[]',
            difficulty TEXT DEFAULT 'beginner',
            estimated_time TEXT DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (topic_id) REFERENCES topics(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS conversations (
            id TEXT PRIMARY KEY NOT NULL,
            topic_id TEXT NOT NULL,
            title TEXT DEFAULT 'New Conversation',
            messages TEXT DEFAULT '[]',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (topic_id) REFERENCES topics(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS bookmarks (
            id TEXT PRIMARY KEY NOT NULL,
            topic_id TEXT NOT NULL,
            concept_id TEXT,
            note TEXT DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (topic_id) REFERENCES topics(id) ON DELETE CASCADE,
            FOREIGN KEY (concept_id) REFERENCES concepts(id) ON DELETE SET NULL
        );

        CREATE TABLE IF NOT EXISTS app_settings (
            id TEXT PRIMARY KEY NOT NULL DEFAULT 'default',
            ai_provider TEXT DEFAULT 'ollama',
            ai_model TEXT DEFAULT 'llama3',
            api_key TEXT DEFAULT '',
            api_endpoint TEXT DEFAULT '',
            theme TEXT DEFAULT 'dark',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        -- Insert default settings if not exists
        INSERT OR IGNORE INTO app_settings (id) VALUES ('default');

        -- Create indexes for performance
        CREATE INDEX IF NOT EXISTS idx_topics_workspace ON topics(workspace_id);
        CREATE INDEX IF NOT EXISTS idx_concepts_topic ON concepts(topic_id);
        CREATE INDEX IF NOT EXISTS idx_relationships_topic ON relationships(topic_id);
        CREATE INDEX IF NOT EXISTS idx_knowledge_cards_topic ON knowledge_cards(topic_id);
        CREATE INDEX IF NOT EXISTS idx_knowledge_cards_concept ON knowledge_cards(concept_id);
        CREATE INDEX IF NOT EXISTS idx_timeline_events_topic ON timeline_events(topic_id);
        CREATE INDEX IF NOT EXISTS idx_notes_topic ON notes(topic_id);
        CREATE INDEX IF NOT EXISTS idx_notes_workspace ON notes(workspace_id);
        CREATE INDEX IF NOT EXISTS idx_flashcards_topic ON flashcards(topic_id);
        CREATE INDEX IF NOT EXISTS idx_learning_paths_topic ON learning_paths(topic_id);
        CREATE INDEX IF NOT EXISTS idx_conversations_topic ON conversations(topic_id);
        CREATE INDEX IF NOT EXISTS idx_bookmarks_topic ON bookmarks(topic_id);
        "
    )?;

    Ok(())
}
