const WS_URL = process.env.NEXT_PUBLIC_WS_BASE_URL;

const getState = () => {
  if (typeof window === "undefined") return null;
  if (!window.__wsState) {
    window.__wsState = {
      socket: null,
      reconnectTimer: null,
      currentUsername: null,
    };
  }
  return window.__wsState;
};

export const connectSocket = (username, onMessage, onOpen, onClose) => {
  const state = getState();
  if (!state) return; // server-side, bail out

  if (state.socket && state.socket.readyState === WebSocket.OPEN)
    return state.socket;

  state.currentUsername = username;
  if (!state.currentUsername) return;

  state.socket = new WebSocket(`${WS_URL}/ws?username=${username}`);

  state.socket.onopen = () => {
    console.log("WebSocket connected");
    if (onOpen) onOpen();
  };

  state.socket.onmessage = (event) => {
    try {
      const data = JSON.parse(event.data);
      if (onMessage) onMessage(data);
    } catch (err) {
      console.error("Invalid JSON:", event.data);
    }
  };

  state.socket.onclose = () => {
    console.log("WebSocket disconnected");
    state.socket = null;
    if (onClose) onClose();
    if (state.currentUsername) {
      state.reconnectTimer = setTimeout(() => {
        console.log("Reconnecting...");
        connectSocket(state.currentUsername, onMessage, onOpen, onClose);
      }, 2000);
    }
  };

  state.socket.onerror = (err) => {
    console.error("WebSocket error:", err);
  };

  return state.socket;
};

export const sendMessage = (data) => {
  const state = getState();
  if (!state?.socket || state.socket.readyState !== WebSocket.OPEN) {
    console.error("Socket not connected");
    return;
  }
  state.socket.send(JSON.stringify(data));
};

export const disconnectSocket = () => {
  const state = getState();
  if (!state) return;
  if (state.reconnectTimer) clearTimeout(state.reconnectTimer);
  if (state.socket) {
    state.socket.close();
    state.socket = null;
  }
  state.currentUsername = null;
};
