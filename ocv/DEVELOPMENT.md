# Development Guide

This document provides instructions for setting up and contributing to the Open Context Vault project.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable channel, 1.75+)
- [Node.js](https://nodejs.org/) (v18+)
- [PostgreSQL](https://www.postgresql.org/download/) (v14+)
- [Docker](https://docs.docker.com/get-docker/) and [Docker Compose](https://docs.docker.com/compose/install/)

## Backend Setup

1. Navigate to the backend directory:
   ```bash
   cd backend
   ```

2. Install Rust dependencies:
   ```bash
   cargo build
   ```

3. Set up the database:
   ```bash
   docker-compose up -d db
   cargo run --bin migrate
   ```

4. Start the backend server:
   ```bash
   cargo run
   ```

The GraphQL API will be available at http://localhost:8000/graphql

## Frontend Setup

1. Navigate to the frontend directory:
   ```bash
   cd frontend
   ```

2. Install dependencies:
   ```bash
   npm install
   ```

3. Start the development server:
   ```bash
   npm run dev
   ```

The consent management UI will be available at http://localhost:3000

## Running Tests

```bash
# Backend tests
cd backend
cargo test

# Frontend tests
cd frontend
npm test
```

## Code Style and Linting

- Backend: We use `rustfmt` and `clippy` for Rust code
  ```bash
  cargo fmt
  cargo clippy
  ```

- Frontend: We use ESLint and Prettier
  ```bash
  npm run lint
  npm run format
  ```

## Branching Strategy

- `main`: Stable production code
- `develop`: Integration branch for features
- `feature/*`: Individual feature branches

All contributions should be made via pull requests to the `develop` branch.
