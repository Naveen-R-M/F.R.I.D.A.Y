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

# Create logs directory
RUN mkdir -p /app/logs

# Create a script to run the app with the correct host binding
RUN echo '#!/bin/bash\n\
sed -i "s/socketio.run(app, debug=True, port=5000)/socketio.run(app, host='"'"'0.0.0.0'"'"', debug=True, port=5000)/" app.py\n\
python -u app.py\n\
' > /app/start.sh && chmod +x /app/start.sh

# Environment variables
ENV PYTHONUNBUFFERED=1
ENV FLASK_ENV=development

# Expose port
EXPOSE 5000

# Start the application with the custom script
CMD ["/app/start.sh"]
