let ws;
let currentAssistantMessage = null;
let isProcessing = false;
let currentToolExecutions = [];

function connect() {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    ws = new WebSocket(`${protocol}//${window.location.host}/ws`);

    ws.onopen = () => {
        console.log('Connected to WebSocket');
        updateStatus('Connected', true);
    };

    ws.onclose = () => {
        console.log('Disconnected from WebSocket');
        updateStatus('Disconnected', false);
        setTimeout(connect, 3000);
    };

    ws.onerror = (error) => {
        console.error('WebSocket error:', error);
        updateStatus('Error', false);
    };

    ws.onmessage = (event) => {
        const data = JSON.parse(event.data);
        handleMessage(data);
    };
}

function handleMessage(data) {
    switch(data.type) {
        case 'connected':
            console.log(data.message);
            break;

        case 'start':
            isProcessing = true;
            updateSendButton();
            if (data.role === 'assistant') {
                currentAssistantMessage = createMessage('assistant', '');
                const typingIndicator = document.createElement('div');
                typingIndicator.className = 'typing-indicator';
                typingIndicator.innerHTML = '<div class="typing-dot"></div><div class="typing-dot"></div><div class="typing-dot"></div>';
                currentAssistantMessage.querySelector('.message-content').appendChild(typingIndicator);
            }
            break;

        case 'chunk':
            if (currentAssistantMessage) {
                const content = currentAssistantMessage.querySelector('.message-content');
                const typingIndicator = content.querySelector('.typing-indicator');
                if (typingIndicator) {
                    typingIndicator.remove();
                }

                const textNode = Array.from(content.childNodes).find(node => node.nodeType === Node.TEXT_NODE);
                if (textNode) {
                    textNode.textContent += data.content;
                } else {
                    content.appendChild(document.createTextNode(data.content));
                }
                scrollToBottom();
            }
            break;

        case 'tool_start':
            currentToolExecutions = data.tools || [];
            if (currentAssistantMessage) {
                const content = currentAssistantMessage.querySelector('.message-content');
                const toolDiv = document.createElement('div');
                toolDiv.className = 'tool-execution';
                toolDiv.id = 'current-tools';
                toolDiv.innerHTML = `<div class="tool-name"><i class="bi bi-gear-fill"></i> Executing tools: ${currentToolExecutions.join(', ')}</div>`;
                content.appendChild(toolDiv);
                scrollToBottom();
            }
            break;

        case 'tool_result':
            if (currentAssistantMessage) {
                const toolsDiv = currentAssistantMessage.querySelector('#current-tools');
                if (toolsDiv) {
                    const resultDiv = document.createElement('div');
                    resultDiv.className = 'tool-result';
                    resultDiv.textContent = `${data.tool}: ${JSON.stringify(data.result)}`;
                    toolsDiv.appendChild(resultDiv);
                    scrollToBottom();
                }
            }
            break;

        case 'end':
            isProcessing = false;
            updateSendButton();
            currentAssistantMessage = null;
            currentToolExecutions = [];
            break;
    }
}

function createMessage(role, content) {
    const messagesDiv = document.getElementById('messages');
    const welcomeMsg = messagesDiv.querySelector('.welcome-message');
    if (welcomeMsg) {
        welcomeMsg.remove();
    }

    const messageDiv = document.createElement('div');
    messageDiv.className = `message ${role}`;

    const avatar = document.createElement('div');
    avatar.className = `message-avatar ${role}-avatar`;
    if (role === 'user') {
        avatar.innerHTML = '<i class="bi bi-person-fill"></i>';
    } else {
        avatar.innerHTML = '<i class="bi bi-robot"></i>';
    }

    const contentDiv = document.createElement('div');
    contentDiv.className = 'message-content';
    if (content) {
        contentDiv.textContent = content;
    }

    messageDiv.appendChild(avatar);
    messageDiv.appendChild(contentDiv);
    messagesDiv.appendChild(messageDiv);

    scrollToBottom();
    return messageDiv;
}

function sendMessage() {
    const input = document.getElementById('message-input');
    const message = input.value.trim();

    if (message && ws && ws.readyState === WebSocket.OPEN && !isProcessing) {
        createMessage('user', message);

        ws.send(JSON.stringify({
            type: 'send_message',
            content: message
        }));

        input.value = '';
        autoResize(input);
    }
}

function sendSuggestion(text) {
    const input = document.getElementById('message-input');
    input.value = text;
    sendMessage();
}

function handleKeyPress(event) {
    if (event.key === 'Enter' && !event.shiftKey) {
        event.preventDefault();
        sendMessage();
    }
}

function autoResize(textarea) {
    textarea.style.height = 'auto';
    textarea.style.height = Math.min(textarea.scrollHeight, 200) + 'px';
}

function scrollToBottom() {
    const messages = document.getElementById('messages');
    messages.scrollTop = messages.scrollHeight;
}

function updateStatus(text, connected) {
    document.getElementById('status-text').textContent = text;
    const dot = document.querySelector('.status-dot');
    dot.style.background = connected ? '#10b981' : '#ef4444';
}

function updateSendButton() {
    const button = document.getElementById('send-button');
    button.disabled = isProcessing;
}

// Initialize when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    connect();
    document.getElementById('message-input').focus();
});