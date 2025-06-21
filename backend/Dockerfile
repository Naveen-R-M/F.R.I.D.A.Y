FROM python:3.9-slim

WORKDIR /app

# Install system dependencies first
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ffmpeg \
    build-essential \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Copy requirements and install Python dependencies
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy the rest of the application code
COPY . .

# Make the start script executable
RUN chmod +x /app/start.sh

# Environment variables
ENV PYTHONUNBUFFERED=1
ENV FLASK_ENV=development

# Expose port
EXPOSE 5000

# Start the application
CMD ["/app/start.sh"]
