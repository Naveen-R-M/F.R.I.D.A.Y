// script.js

// Initialize Socket.IO connection
const socket = io('http://localhost:5000');

// DOM elements
const status = document.getElementById('status');
const conversation = document.getElementById('conversation');
const audioVisualizer = document.getElementById('audioVisualizer');
const visualizerCtx = audioVisualizer.getContext('2d');
const resetButton = document.getElementById('resetButton');

// Global state
let recognition;
let isListening = false;
let isProcessing = false;
let silenceTimer;
let finalTranscript = '';
let isWakeWordActive = false;
let audioContext = null;
let shouldDeactivateAfterAudio = false;
let speechRecognitionAvailable = false;

// --- Audio Unlock Button (required by browsers to enable AudioContext) ---
(function addAudioUnlockButton() {
  const btn = document.createElement('button');
  btn.id = 'audioUnlockBtn';
  btn.textContent = 'Enable Voice Responses';
  btn.style.cssText = `
    position: fixed;
    bottom: 20px;
    right: 20px;
    padding: 12px 20px;
    background: rgba(0,212,255,0.1);
    backdrop-filter: blur(10px);
    border: 1px solid rgba(0,212,255,0.3);
    border-radius: 30px;
    font-size: 14px;
    cursor: pointer;
    z-index: 1000;
  `;
  btn.onclick = async () => {
    try {
      audioContext = new (window.AudioContext || window.webkitAudioContext)();
      await audioContext.resume();
      console.log('AudioContext created and resumed via button.');
      document.body.removeChild(btn);
      // After unlock, kick off recognition
      if (speechRecognitionAvailable) startListening();
      else console.warn('Speech recognition unavailable.');
    } catch (e) {
      console.error('Audio unlock failed:', e);
    }
  };
  document.body.appendChild(btn);
})();

// Initialize speech recognition
function initSpeechRecognition() {
    if (!('webkitSpeechRecognition' in window) && !('SpeechRecognition' in window)) {
        updateStatus('Speech recognition not supported', 'error');
        return false;
    }

    const SpeechRecognition = window.SpeechRecognition || window.webkitSpeechRecognition;
    recognition = new SpeechRecognition();
    recognition.continuous = true;
    recognition.interimResults = true;
    recognition.lang = 'en-US';
    recognition.maxAlternatives = 1;

    recognition.onstart = () => {
        console.log('Speech recognition started');
        if (!isWakeWordActive) {
            updateStatus('Say "Hello Friday" to activate', 'active');
        }
    };

    recognition.onresult = (event) => {
        let interimTranscript = '';
        for (let i = event.resultIndex; i < event.results.length; i++) {
            const transcript = event.results[i][0].transcript;
            if (event.results[i].isFinal) {
                if (!isWakeWordActive &&
                    (transcript.toLowerCase().includes('hello friday') ||
                     transcript.toLowerCase().includes('hey friday'))) {
                    activateFriday();
                    finalTranscript = '';
                    return;
                }
                if (isWakeWordActive) {
                    finalTranscript += transcript + ' ';
                    const lower = transcript.toLowerCase();
                    if (lower.includes('goodbye') || lower.includes('stop listening') ||
                        lower.includes('go to sleep') || lower.includes('deactivate')) {
                        processSpeech('goodbye');
                        return;
                    }
                    resetSilenceTimer();
                }
            } else {
                interimTranscript += transcript;
                if (!isWakeWordActive &&
                    (interimTranscript.toLowerCase().includes('hello friday') ||
                     interimTranscript.toLowerCase().includes('hey friday'))) {
                    activateFriday();
                    return;
                }
                if (isWakeWordActive && interimTranscript.trim()) {
                    resetSilenceTimer();
                }
            }
        }
        if (isWakeWordActive) {
            const currentText = finalTranscript + interimTranscript;
            if (currentText.trim()) {
                updateStatus(`"${currentText.trim()}"`, 'active');
            }
        }
    };

    recognition.onerror = (event) => {
        console.error('Speech recognition error:', event.error);
        if (event.error === 'no-speech') return;
        let errorMessage = 'Speech recognition error';
        if (event.error === 'network') errorMessage = 'Network error - please check your connection';
        else if (event.error === 'not-allowed') errorMessage = 'Microphone access denied';
        else if (event.error === 'audio-capture') errorMessage = 'No microphone found';
        updateStatus(errorMessage, 'error');
        setTimeout(() => startListening(), 3000);
    };

    recognition.onend = () => {
        console.log('Speech recognition ended');
        isListening = false;
        if (isWakeWordActive && finalTranscript.trim() && !isProcessing) {
            processSpeech(finalTranscript.trim());
        }
        if (!isProcessing) {
            setTimeout(() => startListening(), 100);
        }
    };

    return true;
}

// Activate FRIDAY on wake word
function activateFriday() {
    isWakeWordActive = true;
    finalTranscript = '';
    updateStatus('Listening... Speak now', 'active');
    document.querySelector('.voice-visualizer').classList.add('active');
    animateVisualizer();
    playActivationSound();
    resetSilenceTimer();
}

// Reset silence timer
function resetSilenceTimer() {
    clearTimeout(silenceTimer);
    silenceTimer = setTimeout(() => {
        if (finalTranscript.trim() && !isProcessing) {
            processSpeech(finalTranscript.trim());
        } else if (!finalTranscript.trim() && isWakeWordActive) {
            updateStatus('Still listening... speak anytime', 'active');
            silenceTimer = setTimeout(() => {
                if (!finalTranscript.trim() && !isProcessing) {
                    deactivateFriday();
                    updateStatus('Session ended. Say "Hello Friday" to activate');
                }
            }, 30000);
        }
    }, 3000);
}

// Deactivate FRIDAY
function deactivateFriday() {
    isWakeWordActive = false;
    document.querySelector('.voice-visualizer').classList.remove('active');
}

// Process recognized speech
function processSpeech(text) {
    console.log('Processing speech:', text);
    isProcessing = true;
    clearTimeout(silenceTimer);
    deactivateFriday();
    addMessage('user', text);
    socket.emit('text_message', { text });
    updateStatus('Thinking...', 'active');
}

// Start listening
function startListening() {
    if (!speechRecognitionAvailable || isListening) return;
    try {
        isListening = true;
        recognition.start();
    } catch (error) {
        console.error('Error starting recognition:', error);
        isListening = false;
        setTimeout(() => startListening(), 1000);
    }
}

// Play activation tone
function playActivationSound() {
    if (!audioContext) return;
    const oscillator = audioContext.createOscillator();
    const gainNode = audioContext.createGain();
    oscillator.connect(gainNode);
    gainNode.connect(audioContext.destination);
    oscillator.frequency.value = 800;
    gainNode.gain.value = 0.1;
    oscillator.start();
    oscillator.stop(audioContext.currentTime + 0.1);
}

// Socket.IO event handlers
socket.on('connect', () => {
    console.log('Connected to server');
    speechRecognitionAvailable = initSpeechRecognition();
    // Wait for user to unlock audioContext via button
    updateStatus('Click “Enable Voice Responses” to begin', 'active');
});

socket.on('disconnect', () => {
    console.log('Disconnected from server');
    updateStatus('Disconnected', 'error');
});

socket.on('response', async (data) => {
    console.log('Received response:', data);
    addMessage('assistant', data.text);
    const isGoodbye = data.text.toLowerCase().includes('goodbye') ||
                      data.text.toLowerCase().includes('see you') ||
                      data.text.toLowerCase().includes('going to sleep');
    if (data.audio) {
        console.log('Audio data received, length:', data.audio.length);
        shouldDeactivateAfterAudio = isGoodbye;
        await playAudioResponse(data.audio);
    } else {
        console.error('No audio data in response!');
        if (isGoodbye) {
            deactivateFriday();
            updateStatus('Say "Hello Friday" to activate');
            isProcessing = false;
            finalTranscript = '';
        } else {
            handleAudioEnd();
        }
    }
});

// Play TTS audio response
async function playAudioResponse(base64Audio) {
    try {
        if (!audioContext) {
            console.warn('AudioContext not initialized—cannot play audio.');
            return;
        }
        if (audioContext.state === 'suspended') {
            await audioContext.resume();
            console.log('AudioContext resumed');
        }
        if (recognition && isListening) {
            recognition.stop();
            isListening = false;
        }
        const blob = base64ToBlob(base64Audio, 'audio/mp3');
        const url = URL.createObjectURL(blob);
        const audio = new Audio(url);
        audio.volume = 1.0;
        audio.preload = 'auto';
        updateStatus('Speaking...', 'active');
        await audio.play();
        audio.onended = () => {
            URL.revokeObjectURL(url);
            handleAudioEnd();
        };
    } catch (error) {
        console.error('Audio playback error:', error);
        updateStatus('Audio playback failed', 'error');
        handleAudioEnd();
    }
}

// Handle end of audio
function handleAudioEnd() {
    if (shouldDeactivateAfterAudio) {
        shouldDeactivateAfterAudio = false;
        deactivateFriday();
        updateStatus('Say "Hello Friday" to activate');
        isProcessing = false;
        finalTranscript = '';
        return;
    }
    isProcessing = false;
    finalTranscript = '';
    isWakeWordActive = true;
    updateStatus('Listening...', 'active');
    document.querySelector('.voice-visualizer').classList.add('active');
    animateVisualizer();
    startListening();
    resetSilenceTimer();
}

// Convert base64 to Blob
function base64ToBlob(base64, mimeType) {
    const byteChars = atob(base64.replace(/\s/g, ''));
    const byteNumbers = Array.from(byteChars).map(c => c.charCodeAt(0));
    return new Blob([new Uint8Array(byteNumbers)], { type: mimeType });
}

// Add message to chat
function addMessage(sender, text) {
    const div = document.createElement('div');
    div.classList.add('message', sender);
    if (sender === 'assistant') {
        const avatar = document.createElement('div'); avatar.className = 'assistant-avatar';
        avatar.innerHTML = '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="12" cy="12" r="10"></circle><path d="M12 6v6l4 2"></path></svg>';
        div.appendChild(avatar);
    }
    const bubble = document.createElement('div'); bubble.classList.add('message-bubble', sender);
    bubble.innerHTML = text.replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>').replace(/\*(.+?)\*/g, '<em>$1</em>');
    div.appendChild(bubble);
    conversation.appendChild(div);
    setTimeout(() => conversation.scrollTo({ top: conversation.scrollHeight, behavior: 'smooth' }), 100);
}

// Update status line
function updateStatus(msg, cls='') { status.textContent = msg; status.className = 'status'; if (cls) status.classList.add(cls); }

// Visualizer
let anim;
function animateVisualizer() {
    const ctx = visualizerCtx, w = audioVisualizer.width, h = audioVisualizer.height;
    ctx.fillStyle = 'rgba(10,10,10,0.2)'; ctx.fillRect(0,0,w,h);
    for (let i=0;i<50;i++){ const amp=Math.random()*40+10;
        ctx.fillStyle=`hsl(${(i/50)*60+180},100%,${50+(Math.random()*20)}%)`;
        ctx.fillRect(i*w/50,h/2-amp,w/50-1,amp);
        ctx.fillRect(i*w/50,h/2,w/50-1,amp);
        ctx.shadowBlur=10;ctx.shadowColor=`hsl(${(i/50)*60+180},100%,70%)`;
    }ctx.shadowBlur=0;
    if(isWakeWordActive) anim=requestAnimationFrame(animateVisualizer);
}

// Reset conversation
resetButton.addEventListener('click', async ()=>{
    try{const r=await fetch('http://localhost:5000/api/reset-conversation',{method:'POST'});
        if(r.ok){conversation.innerHTML='<div class="welcome-message"><div class="assistant-avatar"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="12" cy="12" r="10"></circle><path d="M12 6v6l4 2"></path></svg></div><div class="message-bubble assistant"><p>Conversation reset. Say <span class="highlight">"Hello Friday"</span> to start a new conversation!</p></div></div>';
            updateStatus('Conversation reset','active');setTimeout(()=>updateStatus('Say "Hello Friday" to activate'),2000);
        }else updateStatus('Reset failed','error');
    }catch(e){console.error(e);updateStatus('Reset failed','error');setTimeout(()=>updateStatus('Say "Hello Friday" to activate'),2000);} 
});
