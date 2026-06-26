# Graph Report - .  (2026-06-26)

## Corpus Check
- Corpus is ~15,795 words - fits in a single context window. You may not need a graph.

## Summary
- 187 nodes · 212 edges · 14 communities detected
- Extraction: 100% EXTRACTED · 0% INFERRED · 0% AMBIGUOUS
- Token cost: 0 input · 0 output
- Edge kinds: contains: 166 · calls: 46


## Input Scope
- Requested: auto
- Resolved: all (source: default-auto)
- Included files: 25 · Candidates: recursive
- Excluded: 0 untracked · 0 ignored · 0 sensitive · 0 missing committed
## God Nodes (most connected - your core abstractions)
1. `renderAll()` - 8 edges
2. `loadTopics()` - 7 edges
3. `sanitize_ai_json()` - 6 edges
4. `loadWorkspaces()` - 6 edges
5. `extract_json()` - 4 edges
6. `fetch_models_from_provider()` - 4 edges
7. `loadTopicData()` - 4 edges
8. `renderTree()` - 4 edges
9. `renderCurrentFlashcard()` - 4 edges
10. `call_ai()` - 3 edges

## Surprising Connections (you probably didn't know these)
- None detected - all connections are within the same source files.

## Communities

### Community 0 - "Community 0"
Cohesion: 0.05
Nodes (22): AIExplorationResult, AppSettings, Bookmark, ChatMessage, Concept, ConceptInput, Conversation, Flashcard (+14 more)

### Community 1 - "Community 1"
Cohesion: 0.13
Nodes (21): buildStrictTree(), data, expandConcept(), loadTopicData(), nextFlashcard(), prevFlashcard(), quizQuestions, REL_COLORS (+13 more)

### Community 2 - "Community 2"
Cohesion: 0.17
Nodes (14): createWorkspace(), deleteTopic(), deleteWorkspace(), filterByWorkspace(), loadTopics(), loadWorkspaces(), moveTopic(), openWorkspaceView() (+6 more)

### Community 3 - "Community 3"
Cohesion: 0.17
Nodes (3): API, esc(), showToast()

### Community 4 - "Community 4"
Cohesion: 0.17
Nodes (8): ExpandRequest, ExplainRequest, explore(), ExploreRequest, ModelsRequest, QuizRequest, save_exploration_results(), TutorRequest

### Community 5 - "Community 5"
Cohesion: 0.54
Nodes (7): attempt_fix_quotes(), extract_json(), extract_json_array(), regex_lite_replace(), safe_parse_json(), sanitize_ai_json(), strip_code_fences()

### Community 6 - "Community 6"
Cohesion: 0.39
Nodes (7): fetch_models_from_provider(), fetch_ollama_models(), get_all_providers(), get_provider(), ModelInfo, ProviderConfig, test_provider_connection()

### Community 7 - "Community 7"
Cohesion: 0.25
Nodes (3): CreateWorkspace, DeleteQuery, UpdateWorkspace

### Community 8 - "Community 8"
Cohesion: 0.38
Nodes (5): AIConfig, call_ai(), call_anthropic(), call_openai_compatible(), ChatMessage

### Community 9 - "Community 9"
Cohesion: 0.29
Nodes (2): CreateConcept, UpdateConcept

### Community 10 - "Community 10"
Cohesion: 0.29
Nodes (2): CreateNote, UpdateNote

### Community 11 - "Community 11"
Cohesion: 0.29
Nodes (2): CreateTopic, UpdateTopicWorkspace

### Community 13 - "Community 13"
Cohesion: 1.00
Nodes (3): get_db_path(), get_workspace_dir(), init_db()

### Community 14 - "Community 14"
Cohesion: 0.50
Nodes (1): UpdateSettings

## Knowledge Gaps
- **49 isolated node(s):** `AIConfig`, `ChatMessage`, `ProviderConfig`, `ModelInfo`, `Workspace` (+44 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **Thin community `Community 9`** (2 nodes): `CreateConcept`, `UpdateConcept`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 10`** (2 nodes): `CreateNote`, `UpdateNote`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 11`** (2 nodes): `CreateTopic`, `UpdateTopicWorkspace`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 14`** (1 nodes): `UpdateSettings`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **What connects `AIConfig`, `ChatMessage`, `ProviderConfig` to the rest of the system?**
  _49 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Community 0` be split into smaller, more focused modules?**
  _Cohesion score 0.05 - nodes in this community are weakly interconnected._
- **Should `Community 1` be split into smaller, more focused modules?**
  _Cohesion score 0.13333333333333333 - nodes in this community are weakly interconnected._