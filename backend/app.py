from flask import Flask, request, jsonify, send_file
from flask_cors import CORS
from flask_socketio import SocketIO, emit

from context_provider import ContextProvider

import os
import json
import base64
import io
import re
import edge_tts
import asyncio
import google.generativeai as genai
from dotenv import load_dotenv
import logging
from datetime import datetime
import sys
import codecs

# Create logs directory if it doesn't exist
logs_dir = os.path.join(os.path.dirname(os.path.dirname(__file__)), 'logs')
os.makedirs(logs_dir, exist_ok=True)

# Set up logging with UTF-8 encoding
log_filename = os.path.join(logs_dir, f'friday_{datetime.now().strftime("%Y%m%d_%H%M%S")}.log')

# Configure file handler with UTF-8 encoding
file_handler = logging.FileHandler(log_filename, encoding='utf-8')
file_handler.setLevel(logging.INFO)
file_handler.setFormatter(logging.Formatter('%(asctime)s - %(name)s - %(levelname)s - %(message)s'))

# Configure console handler with UTF-8 encoding
console_handler = logging.StreamHandler(sys.stdout)
console_handler.setLevel(logging.INFO)
console_handler.setFormatter(logging.Formatter('%(asctime)s - %(name)s - %(levelname)s - %(message)s'))

# For Windows console, ensure UTF-8 output
if sys.platform == 'win32':
    sys.stdout = codecs.getwriter('utf-8')(sys.stdout.buffer, 'strict')
    sys.stderr = codecs.getwriter('utf-8')(sys.stderr.buffer, 'strict')

# Set up root logger
logger = logging.getLogger(__name__)
logger.setLevel(logging.INFO)
logger.addHandler(file_handler)
logger.addHandler(console_handler)

# Log startup
logger.info("=" * 50)
logger.info("F.R.I.D.A.Y Backend Starting...")
logger.info("=" * 50)

# Load environment variables
load_dotenv()

app = Flask(__name__)
CORS(app)
socketio = SocketIO(app, cors_allowed_origins="*")

# Configure Gemini
api_key = os.getenv('GEMINI_API_KEY')
if api_key:
    genai.configure(api_key=api_key)
    logger.info("Gemini API key configured successfully")
else:
    logger.error("No Gemini API key found in .env file!")

# Initialize Gemini model
try:
    gemini_model = genai.GenerativeModel('gemini-2.5-flash')
    logger.info("Gemini model initialized")
except Exception as e:
    logger.error(f"Failed to initialize Gemini model: {e}")
    gemini_model = None

# Voice configuration for Edge-TTS
VOICE = "en-US-AriaNeural"

class ConversationManager:
    def __init__(self):
        self.chat = None
        self.messages = []
        self.system_prompt = """
        You are FRIDAY, a helpful AI assistant that personalizes replies using the user’s current context.
        Whenever you receive a `[CONTEXT]` JSON it may include:
        • weather, location, upcoming_events, user_name, preferences,
        • AND/OR a `nearby_stores` list of {name, address, latitude, longitude, distance_km, eta_min}.
        
        If `nearby_stores` is present, you MUST:
            1) Pick the top store (or list the top 3 if user asked for “stores”),
            2) Mention its name, address, how far it is (`distance_km`), and how long it takes (`eta_min`),
            3) Offer to navigate there or place an online order.
            
        Only ask follow-up questions if the context lacks the needed information.
        """

    
    def start_chat(self):
        try:
            self.chat = gemini_model.start_chat(history=[])
            self.chat.send_message(self.system_prompt)
            logger.info("New chat session started")
        except Exception as e:
            logger.error(f"Failed to start chat: {e}")
        
    def add_message(self, role, content):
        self.messages.append({"role": role, "content": content})
        if len(self.messages) > 10:
            self.messages = self.messages[-10:]
    
    def get_response(self, user_input):
        try:
            if not self.chat:
                self.start_chat()
            
            self.add_message("user", user_input)
            logger.info(f"User: {user_input}")
            
            # Check for goodbye messages
            if user_input.lower() in ['goodbye', 'bye', 'stop listening', 'go to sleep', 'deactivate']:
                assistant_response = "Goodbye! I'm going to sleep now. Just say 'Hello Friday' when you need me again."
            else:
                # 1️⃣ detect purchase intent
                purchase_query = None
                if user_input.lower().startswith("i'd like to buy") or "buy" in user_input.lower():
                    purchase_query = user_input
                    
                # 2️⃣ build context, passing purchase_query if any
                ctx = context_provider.build_context(purchase_query=purchase_query)
                prompt = (
                    f"[CONTEXT] {json.dumps(ctx)}\n\n"
                    f"[USER] {user_input}"
                )
                
                response = self.chat.send_message(prompt)

                assistant_response = response.text
            
            self.add_message("assistant", assistant_response)
            logger.info(f"Assistant: {assistant_response}")
            return assistant_response
            
        except Exception as e:
            logger.error(f"Error getting Gemini response: {e}")
            self.start_chat()
            try:
                response = self.chat.send_message(user_input)
                return response.text
            except Exception as e2:
                logger.error(f"Failed to get response after retry: {e2}")
                return "I'm sorry, I encountered an error processing your request. Please try again."

conversation_manager = ConversationManager()
context_provider = ContextProvider(config_path="config.json")

def detect_emotion(text):
    """Detect emotion from text to apply appropriate voice style"""
    text_lower = text.lower()
    
    if any(word in text_lower for word in ['excited', 'amazing', 'wonderful', 'fantastic', 'great news', '!']):
        return 'cheerful'
    elif any(word in text_lower for word in ['sorry', 'unfortunately', 'sad', 'difficult']):
        return 'empathetic'
    elif any(word in text_lower for word in ['?', 'how', 'what', 'why', 'when', 'where']):
        return 'friendly'
    elif any(word in text_lower for word in ['urgent', 'important', 'warning', 'alert']):
        return 'serious'
    else:
        return 'chat'

async def generate_speech(text, emotion='chat'):
    """Generate speech with emotion using Edge-TTS"""
    voice_map = {
        'cheerful': "en-US-AriaNeural",
        'empathetic': "en-US-JennyNeural",
        'friendly': "en-US-AriaNeural",
        'serious': "en-US-SaraNeural",
        'chat': "en-US-AriaNeural"
    }
    
    selected_voice = voice_map.get(emotion, VOICE)
    
    # Create communicate instance with plain text
    communicate = edge_tts.Communicate(text, selected_voice)
    
    # Generate audio to bytes
    audio_data = b""
    async for chunk in communicate.stream():
        if chunk["type"] == "audio":
            audio_data += chunk["data"]
    
    logger.info(f"Generated audio data size: {len(audio_data)} bytes")
    return audio_data

def clean_text_for_speech(text):
    """Clean text for speech synthesis"""
    # Remove bold markdown
    text = text.replace('**', '')
    # Remove italic markdown
    text = text.replace('*', '')
    # Remove other common markdown
    text = text.replace('##', '').replace('###', '')
    
    # Remove emojis
    emoji_pattern = re.compile("["
                               u"\U0001F600-\U0001F64F"
                               u"\U0001F300-\U0001F5FF"
                               u"\U0001F680-\U0001F6FF"
                               u"\U0001F1E0-\U0001F1FF"
                               u"\U00002702-\U000027B0"
                               u"\U000024C2-\U0001F251"
                               u"\U0001F900-\U0001F9FF"
                               u"\U00002600-\U000026FF"
                               u"\U00002700-\U000027BF"
                               "]+", flags=re.UNICODE)
    text = emoji_pattern.sub('', text)
    
    # Clean up any double spaces
    return ' '.join(text.split())

@app.route('/')
def index():
    return jsonify({"status": "FRIDAY Backend is running"})

@app.route('/api/health', methods=['GET'])
def health_check():
    health_status = {
        "status": "healthy",
        "gemini": "configured" if os.getenv('GEMINI_API_KEY') else "not configured",
        "mode": "hands-free voice recognition",
        "voice": "emotional TTS enabled"
    }
    logger.info(f"Health check: {health_status}")
    return jsonify(health_status)

@socketio.on('connect')
def handle_connect():
    logger.info('Client connected')
    emit('connected', {'data': 'Connected to FRIDAY'})

@socketio.on('disconnect')
def handle_disconnect():
    logger.info('Client disconnected')

@socketio.on('text_message')
def handle_text_message(data):
    """Handle text messages from client-side speech recognition"""
    try:
        text = data.get('text', '').strip()
        
        if not text:
            emit('error', {'message': 'No text received'})
            return
        
        logger.info(f"Received text message: {text}")
        
        # Get Gemini response
        response_text = conversation_manager.get_response(text)
        
        # Clean up text for speech
        speech_text = clean_text_for_speech(response_text)
        
        # Detect emotion from the response
        emotion = detect_emotion(response_text)
        logger.info(f"Detected emotion: {emotion}")
        
        # Convert response to speech with emotion
        try:
            # Run async function in sync context
            loop = asyncio.new_event_loop()
            asyncio.set_event_loop(loop)
            audio_data = loop.run_until_complete(generate_speech(speech_text, emotion))
            loop.close()
            
            audio_base64 = base64.b64encode(audio_data).decode('utf-8')
            logger.info(f"Text-to-speech conversion successful with {emotion} style")
            logger.info(f"Base64 audio length: {len(audio_base64)} characters")
            
            # Verify base64 encoding
            try:
                test_decode = base64.b64decode(audio_base64)
                if len(test_decode) == len(audio_data):
                    logger.info("Base64 encoding verified successfully")
                else:
                    logger.error(f"Base64 encoding mismatch: original {len(audio_data)} vs decoded {len(test_decode)}")
            except Exception as e:
                logger.error(f"Base64 verification failed: {e}")
        except Exception as e:
            logger.error(f"Text-to-speech failed: {e}")
            audio_base64 = None
        
        # Send response back to client
        emit('response', {
            'text': response_text,
            'audio': audio_base64,
            'emotion': emotion
        })
        
        logger.info("Response sent successfully")
        
    except Exception as e:
        logger.error(f"Error processing text message: {e}", exc_info=True)
        emit('error', {'message': f'Error: {str(e)}'})

@app.route('/api/chat', methods=['POST'])
def chat():
    """REST API endpoint for text chat"""
    try:
        data = request.json
        text = data.get('text', '').strip()
        
        if not text:
            return jsonify({'error': 'No text provided'}), 400
        
        logger.info(f"Chat API request: {text}")
        
        response_text = conversation_manager.get_response(text)
        
        return jsonify({
            'success': True,
            'response': response_text
        })
        
    except Exception as e:
        logger.error(f"Chat API error: {e}")
        return jsonify({'success': False, 'error': str(e)}), 500

@app.route('/api/reset-conversation', methods=['POST'])
def reset_conversation():
    """Reset the conversation history"""
    conversation_manager.messages = []
    conversation_manager.chat = None
    logger.info("Conversation reset by user")
    return jsonify({'status': 'Conversation reset'})

@app.route('/api/text-to-speech', methods=['POST'])
def text_to_speech():
    """Convert text to speech using Edge-TTS"""
    try:
        data = request.json
        text = data.get('text', '')
        
        if not text:
            return jsonify({'error': 'No text provided'}), 400
        
        logger.info(f"Text-to-speech request: {text[:50]}...")
        
        # Clean text for speech
        speech_text = clean_text_for_speech(text)
        
        # Generate speech using Edge-TTS
        loop = asyncio.new_event_loop()
        asyncio.set_event_loop(loop)
        audio_data = loop.run_until_complete(generate_speech(speech_text, 'friendly'))
        loop.close()
        
        # Return as audio file
        audio_buffer = io.BytesIO(audio_data)
        audio_buffer.seek(0)
        
        return send_file(
            audio_buffer,
            mimetype='audio/mp3',
            as_attachment=True,
            download_name='response.mp3'
        )
    except Exception as e:
        logger.error(f"Text-to-speech error: {e}")
        return jsonify({'error': str(e)}), 500

if __name__ == '__main__':
    logger.info("Starting Flask application...")
    logger.info(f"Log file: {log_filename}")
    logger.info("Mode: Hands-free voice recognition with emotional TTS")
    logger.info(f"Voice: {VOICE}")
    
    if not os.getenv('GEMINI_API_KEY'):
        logger.warning("WARNING: No Gemini API key found! Please add it to .env file")
    
    socketio.run(app, host='0.0.0.0', debug=True, port=5000)