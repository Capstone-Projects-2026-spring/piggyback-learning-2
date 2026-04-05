
# Piggyback Learning (backend)

A backend web application built with [Loco.rs](https://loco.rs/) (Axum), [SeaORM](https://www.sea-ql.org/SeaORM/), and [Vosk](https://alphacephei.com/vosk/) for offline speech recognition.

----------

## Table of Contents

-   [Prerequisites](https://claude.ai/chat/24c2c6ac-fd6f-46a3-9336-b8177a59fa18#prerequisites)
-   [Installing Rust](https://claude.ai/chat/24c2c6ac-fd6f-46a3-9336-b8177a59fa18#installing-rust)
-   [Installing Loco CLI](https://claude.ai/chat/24c2c6ac-fd6f-46a3-9336-b8177a59fa18#installing-loco-cli)
-   [Setting Up Vosk](https://claude.ai/chat/24c2c6ac-fd6f-46a3-9336-b8177a59fa18#setting-up-vosk)
-   [Project Setup](https://claude.ai/chat/24c2c6ac-fd6f-46a3-9336-b8177a59fa18#project-setup)
-   [Configuration](https://claude.ai/chat/24c2c6ac-fd6f-46a3-9336-b8177a59fa18#configuration)
-   [Database Migrations](https://claude.ai/chat/24c2c6ac-fd6f-46a3-9336-b8177a59fa18#database-migrations)
-   [Running the Project](https://claude.ai/chat/24c2c6ac-fd6f-46a3-9336-b8177a59fa18#running-the-project)
-   [Testing](https://claude.ai/chat/24c2c6ac-fd6f-46a3-9336-b8177a59fa18#testing)
-   [Useful Commands](https://claude.ai/chat/24c2c6ac-fd6f-46a3-9336-b8177a59fa18#useful-commands)

----------

## Prerequisites

Before you begin, ensure you have the following:

-   **Rust** (stable toolchain, 1.75+) — see [Installing Rust](https://claude.ai/chat/24c2c6ac-fd6f-46a3-9336-b8177a59fa18#installing-rust)
-   **SQLite** — usually pre-installed on macOS and Linux; Windows users can download it from https://www.sqlite.org/download.html
-   **Vosk v0.3.45 native library** — see [Setting Up Vosk](https://claude.ai/chat/24c2c6ac-fd6f-46a3-9336-b8177a59fa18#setting-up-vosk)
-   `pkg-config` and platform build tools (see platform notes below)

### Platform Build Tools

**Linux (Debian/Ubuntu):**

```bash
sudo apt update && sudo apt install -y build-essential pkg-config libssl-dev libsqlite3-dev

```

**macOS:**

```bash
xcode-select --install

```

**Windows:**

Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with the "Desktop development with C++" workload selected.

----------

## Installing Rust

Rust is installed via `rustup`, the official Rust toolchain manager.

Follow the official installation guide for your platform:

-   **All platforms:** https://www.rust-lang.org/tools/install
-   **rustup documentation:** https://rust-lang.github.io/rustup/

After installation, verify it worked:

```bash
rustc --version
cargo --version

```

To keep Rust up to date:

```bash
rustup update stable

```

----------

## Installing Loco CLI

The `loco` CLI is used to manage and run your application.

```bash
cargo install loco-cli

```

Verify the installation:

```bash
loco --version

```

> Full Loco documentation: https://loco.rs/docs/

----------

## Setting Up Vosk

Vosk is an offline speech recognition toolkit. The Rust bindings require the native Vosk shared library (v0.3.45) to be present on your system.

### 1. Download the Vosk v0.3.45 Library

Download the pre-built library for your platform from the official v0.3.45 release page:

-   **Vosk v0.3.45 release:** https://github.com/alphacep/vosk-api/releases/tag/v0.3.45


| Platform          | File to download                 |
|-------------------|----------------------------------|
| Linux x86_64      | `vosk-linux-x86_64-0.3.45.zip`   |
| macOS (Universal) | `vosk-osx-universal-0.3.45.zip`  |
| Windows x86_64    | `vosk-win64-0.3.45.zip`          |



### 2. Install the Library

**Linux:**
```bash
# Extract and move the shared library to a system path
unzip vosk-linux-x86_64-*.zip
sudo cp vosk-linux-x86_64-*/libvosk.so /usr/local/lib/
sudo ldconfig
```

**macOS:**
```bash
unzip vosk-osx-universal-*.zip
cp vosk-osx-universal-*/libvosk.dylib /usr/local/lib/
```

**Windows:**

Extract the zip and place `libvosk.dll` in your project directory or add it to your system `PATH`.

### 3. Set the Library Path (if needed)

If the linker cannot find the library at build time, set the path explicitly:

```bash
# Linux / macOS
export VOSK_LIB_PATH=/usr/local/lib

# Or point to the extracted directory directly
export VOSK_LIB_PATH=/path/to/vosk-linux-x86_64-0.3.45
```

Add this to your shell profile (`~/.bashrc`, `~/.zshrc`, etc.) to persist it.


### 4. Download a Vosk Language Model

Vosk requires a language model to perform speech recognition. Download a model for your target language from the official model list:

-   **Model list:** https://alphacephei.com/vosk/models

Extract the model **directly into a `vosk/` folder inside `backend/`**:

**Linux / macOS:**

```bash
mkdir -p vosk
wget https://alphacephei.com/vosk/models/vosk-model-small-en-us-0.15.zip
unzip vosk-model-small-en-us-0.15.zip -d vosk/

```

**Windows (PowerShell):**

```powershell
New-Item -ItemType Directory -Force -Path vosk
Invoke-WebRequest -Uri https://alphacephei.com/vosk/models/vosk-model-small-en-us-0.15.zip -OutFile vosk-model-small-en-us-0.15.zip
Expand-Archive vosk-model-small-en-us-0.15.zip -DestinationPath vosk

```

This produces the following structure inside `backend/`:

```
backend/
├── vosk/
│   └── vosk-model-small-en-us-0.15/
│       ├── am/
│       ├── conf/
│       └── ...
└── ...

```

The path `vosk/vosk-model-small-en-us-0.15` is what you'll set as `VOSK_DIR` in your `.env`.

----------

## Project Setup

### 1. Clone the Repository

```bash
git clone https://github.com/Capstone-Projects-2026-spring/piggyback-learning-2.git
cd piggyback-learning-2/backend

```

### 2. Install Dependencies

Cargo handles Rust dependencies automatically on build. To pre-fetch them:

```bash
cargo fetch

```

----------

## Configuration

Loco uses environment-based YAML config files found in `config/`.

```
config/
├── development.yaml   # Used when LOCO_ENV=development (default)
├── production.yaml    # Used when LOCO_ENV=production
└── test.yaml          # Used when LOCO_ENV=test

```

### Database

This project uses **SQLite**. No database server setup is required — SQLite will create the `.sqlite3` file automatically on first run. The URI is already configured in the config files:

```yaml
database:
  uri: sqlite://backend_development.sqlite?mode=rwc
  enable_logging: false
  min_connections: 1
  max_connections: 1

```

### Environment Variables

Copy the example env file and fill in the paths for your platform:

```bash
cp env.example .env

```

Edit `.env` and fill in your values:

```env
JWT_SECRET=your_jwt_secret_here
JWT_EXPIRATION=3600
OPENAI_API_KEY=your_openai_api_key_here
VOSK_DIR="vosk/vosk-model-small-en-us-0.15"

```

`VOSK_DIR` should be the path to the extracted model folder relative to `backend/`. If you followed the steps above, the value will be `vosk/vosk-model-small-en-us-0.15`.

> The `.env` file is gitignored and should never be committed.

----------

## Database Migrations

Run all pending migrations before starting the server for the first time, and after pulling changes that include new migrations.

### Run All Pending Migrations

```bash
cargo loco db migrate

```

### Check Migration Status

```bash
cargo loco db status

```

### Roll Back the Last Migration

```bash
cargo loco db down

```

----------

## Running the Project

### Development

Start the Loco development server:

```bash
cargo loco start

```

The server starts at [http://localhost:5150](http://localhost:5150/) by default (configurable in `config/development.yaml`).

### Production

Build an optimized release binary:

```bash
cargo build --release

```

Run the release binary:

```bash
LOCO_ENV=production ./target/release/backend-cli start

```

----------

## Testing

Run the full test suite with:

```bash
cargo test

```

To run a specific test or module:

```bash
# Run tests matching a name pattern
cargo test <test_name>

# Run tests in a specific integration test file
cargo test --test <integration_test_file>

```

> Tests run under `LOCO_ENV=test` and use a separate SQLite database, so they will not affect your development data.

----------

## Useful Commands

| Command                                | Description                       |
|----------------------------------------|-----------------------------------|
| `cargo loco start`                     | Start the development server      |
| `cargo loco db migrate`                | Apply all pending migrations      |
| `cargo loco db down`                   | Roll back the last migration      |
| `cargo loco db status`                 | Show migration status             |
| `cargo loco generate model <name>`     | Scaffold a new model + migration  |
| `cargo loco generate controller <name>`| Scaffold a new controller         |
| `cargo loco routes`                    | List all registered routes        |
| `cargo build --release`                | Build optimized production binary |
| `cargo test`                           | Run all tests                     |


----------

## Project Structure

```
backend/
├── config/                        	# Environment config files
│   ├── development.yaml
│   ├── production.yaml
│   └── test.yaml
├── migration/                     	# Database migration files
│   └── src/
├── vosk/                          	# Vosk language model (not committed to git)
│   └── vosk-model-small-en-us-0.15/
├── src/
│   ├── bin/               			    # Main
│   ├── controllers/             	  # Axum route handlers
│   ├── models/                   	# SeaORM models
│   ├── utils/                    	# Utility functions
│ 		├── voice/					          # Voice Utility functions
│   	├── download.rs  			        # Download YouTube video
│   	├── openai.rs  				        # Question generation through OpenAI
│   	└── structs.rs  			        # Commonly used structs
│   ├── app.rs                     	# App bootstrap and router
│   ├── lib.rs                     	# All imports and exports
│   └── openapi.rs   				        # Docs provided through RapiDoc
├── tests/                         	# Integration tests
├── .env                           	# Local environment variables (not committed)
├── Cargo.toml
└── README.md

```

----------

## Learn More

-   [Loco.rs Documentation](https://loco.rs/docs/)
-   [Loco.rs GitHub](https://github.com/loco-rs/loco)
-   [SeaORM Documentation](https://www.sea-ql.org/SeaORM/docs/introduction/sea-orm/)
-   [Vosk API](https://alphacephei.com/vosk/)
-   [Vosk v0.3.45 Release](https://github.com/alphacep/vosk-api/releases/tag/v0.3.45)
-   [Vosk Rust Bindings](https://crates.io/crates/vosk)
-   [Axum Documentation](https://docs.rs/axum/latest/axum/)
