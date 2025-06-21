# Open Context Vault (OCV)

An open-source personal data layer that enables users to maintain ownership and control of their contextual data while interacting with various AI systems.

## Overview

OCV is a secure, privacy-focused service that allows users to:
- Store encrypted profile and history shards
- Control which AI services can access their data
- Share specific slices of context through revocable tokens
- Maintain a comprehensive audit log of data access

## Key Components

- **Core Vault Service**: Rust-based backend with GraphQL API
- **Consent Management**: User-controlled permissions with audit logging
- **Vector Storage**: Semantic search and retrieval of context shards
- **End-to-End Encryption**: Zero-knowledge architecture for maximum privacy

## Project Structure

- `/backend`: Rust-based core services
- `/frontend`: Consent management UI components
- `/sdks`: Client libraries for integration

## Getting Started

See the [Development Guide](./DEVELOPMENT.md) for setup instructions.

## License

Open Context Vault is open source software [licensed as MIT](./LICENSE).
