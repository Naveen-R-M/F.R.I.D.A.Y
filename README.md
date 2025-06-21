# F.R.I.D.A.Y with Open Context Vault

This project integrates F.R.I.D.A.Y (a voice assistant) with Open Context Vault (OCV), a secure personal data layer that uses Basic.tech's mem0 for storing user context.

## Project Structure

- `/backend` - F.R.I.D.A.Y backend (Flask)
- `/frontend` - F.R.I.D.A.Y frontend
- `/ocv` - Open Context Vault implementation
  - `/backend` - OCV backend with mem0 integration
  - `/frontend` - Consent management UI

## Architecture

In this setup:

1. **F.R.I.D.A.Y** serves as the primary voice interface for users
2. **OCV** provides a secure data layer for storing user context
3. **mem0** is used as the underlying memory storage

F.R.I.D.A.Y can request access to user preferences and history stored in OCV, enabling personalized interactions while maintaining user privacy and control.

## Getting Started with Docker

### Prerequisites

- [Docker](https://docs.docker.com/get-docker/) and [Docker Compose](https://docs.docker.com/compose/install/)
- Basic.tech mem0 API key

### Configuration

1. Create a `.env` file in the root directory with your Basic.tech mem0 API key:
   ```
   BASIC_MEM0_API_KEY=your_api_key_here
   ```

### Running with Docker

```bash
# Start all services
docker-compose up

# Run in detached mode
docker-compose up -d

# Stop all services
docker-compose down
```

This will start:
- F.R.I.D.A.Y backend on port 5000
- F.R.I.D.A.Y frontend on port 8080
- OCV backend on port 8000
- OCV consent management UI on port 3000

## Running Without Docker

### Running the F.R.I.D.A.Y Backend

```bash
cd backend
pip install -r requirements.txt
python app.py
```

### Running the F.R.I.D.A.Y Frontend

```bash
cd frontend
python -m http.server 8080
```

### Running the OCV Backend

```bash
cd ocv/backend
cargo run
```

### Running the OCV Consent UI

```bash
cd ocv/frontend/public
python -m http.server 3000
```

## Accessing Services

- F.R.I.D.A.Y Voice Assistant: http://localhost:8080
- OCV Consent Management: http://localhost:3000

To access these services from another computer on your network, replace "localhost" with your computer's IP address.

## Integration Flow

1. User speaks to F.R.I.D.A.Y through the web interface
2. F.R.I.D.A.Y backend processes the speech and identifies the intent
3. If needed, F.R.I.D.A.Y requests access to user context from OCV
4. OCV validates the access permission and provides the requested data
5. F.R.I.D.A.Y uses the context to provide personalized responses
6. Any new context generated during the interaction can be stored back in OCV

This architecture gives users control over their data while allowing F.R.I.D.A.Y to maintain context across different sessions.
