#!/bin/bash
# Find and replace the socketio.run line to ensure it binds to 0.0.0.0
sed -i "s/socketio.run(app, debug=True, port=5000)/socketio.run(app, host='0.0.0.0', debug=True, port=5000)/" app.py
# Run the application
python -u app.py
