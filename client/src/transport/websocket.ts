
    const WS_BASE_URL = 'ws://127.0.0.1:8080/ws';
    
  export const transportWebSocket = {
  // WebSocket Connection Service
  connectWebSocket(
    onMessage: (data: any) => void,
    onStatusChange: (connected: boolean) => void
  ): WebSocket {
    const socket = new WebSocket(WS_BASE_URL);

    socket.onopen = () => {
      console.log('[WS] Connected to Barberaus server');
      onStatusChange(true);
    };

    socket.onmessage = (event) => {
      try {
        const parsed = JSON.parse(event.data);
        onMessage(parsed);
      } catch (err) {
        console.error('[WS] Failed to parse message:', event.data);
      }
    };

    socket.onerror = (err) => {
      console.error('[WS] Error:', err);
      onStatusChange(false);
    };

    socket.onclose = () => {
      console.warn('[WS] Connection closed. Retrying in 3s...');
      onStatusChange(false);
      setTimeout(() => this.connectWebSocket(onMessage, onStatusChange), 3000);
    };

    return socket;
  },
};