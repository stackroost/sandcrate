# Sandcrate

A plugin management system with a Rust backend and React frontend.

## Components

- **Backend** (`sandcrate-backend/`): Rust API server with plugin execution capabilities
- **Frontend** (`sandcrate-react/`): React web application with authentication and plugin management
- **CLI** (`sandcrate-cli/`): Command-line interface
- **Plugin SDK** (`sandcrate-plugin/`): Plugin development kit

## Quick Start

### Backend
```bash
cd sandcrate-backend
cargo run
```
Server runs on `http://localhost:3000`

### Frontend
```bash
cd sandcrate-react
npm install
npm run dev
```
App runs on `http://localhost:5173`

## Features

- Plugin execution with WASM support
- User authentication
- Web-based plugin management interface
- RESTful API

## Tech Stack

- **Backend**: Rust, Axum, Tokio, Wasmtime
- **Frontend**: React, TypeScript, Tailwind CSS, Vite
- **Authentication**: JWT, PAM
