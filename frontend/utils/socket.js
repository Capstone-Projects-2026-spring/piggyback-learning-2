let socket = null;
let reconnectTimer = null;
let currentUsername = null;

const WS_URL = process.env.NEXT_PUBLIC_WS_BASE_URL;

export const connectSocket = (username, onMessage, onOpen, onClose) => {
  if (socket && socket.readyState === WebSocket.OPEN) return socket;

  currentUsername = username;
  if (!currentUsername) return;

  socket = new WebSocket(`${WS_URL}/ws?username=${username}`);

  socket.onopen = () => {
    console.log("WebSocket connected");
    if (onOpen) onOpen();
  };

  socket.onmessage = (event) => {
    try {
      const data = JSON.parse(event.data);
      if (onMessage) onMessage(data);
    } catch (err) {
      console.error("Invalid JSON:", event.data);
    }
  };

  socket.onclose = () => {
    console.log("WebSocket disconnected");
    socket = null;
    if (onClose) onClose();

    if (currentUsername) {
      reconnectTimer = setTimeout(() => {
        console.log("Reconnecting...");
        connectSocket(currentUsername, onMessage, onOpen, onClose);
      }, 2000);
    }
  };

  socket.onerror = (err) => {
    console.error("WebSocket error:", err);
  };

  return socket;
};

export const sendMessage = (data) => {
  if (!socket || socket.readyState !== WebSocket.OPEN) {
    console.error("Socket not connected");
    return;
  }

  socket.send(JSON.stringify(data));
};

export const disconnectSocket = () => {
  if (reconnectTimer) clearTimeout(reconnectTimer);
  if (socket) {
    socket.close();
    socket = null;
  }
  currentUsername = null;
};
