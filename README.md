# Topic Explorer

AI-powered Topic Explorer — learn anything with intelligent concept mapping.

## Features

* **AI-Powered Concept Mapping:** Intelligently explore and connect topics.
* **Workspaces:** Organize your learning materials into separate workspaces.
* **Topics & Concepts:** Drill down into specific subjects and understand their core concepts.
* **Notes:** Take detailed notes as you learn.
* **Local-first Data:** Stores all data locally using SQLite for speed and privacy.
* **Cross-Platform:** Available on Windows, macOS, and Linux.

## Getting Started

### Prerequisites

* [Rust](https://www.rust-lang.org/tools/install) (if building from source)

### Running Locally

1. Clone the repository:
   ```bash
   git clone https://github.com/your-username/rust-ai-topic-explorer.git
   cd rust-ai-topic-explorer
   ```

2. Run the server:
   ```bash
   cargo run
   ```

3. Open your web browser and navigate to `http://127.0.0.1:3080`.

## Automated Builds (CI/CD)

This project uses GitHub Actions to automatically build release binaries across platforms:
- Windows Executable (`.exe`)
- macOS App Bundle (`.app`) for Intel & Apple Silicon
- Linux Debian Package (`.deb`)

To trigger a release build, simply push a new tag matching `v*.*.*` (e.g., `v0.1.0`), or create a new Release in the GitHub UI and create the tag there. The release workflow will build and attach the artifacts automatically.