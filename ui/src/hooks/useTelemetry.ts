import { useState, useEffect } from 'react';

export interface TelemetryEvent {
  node_id: string;
  status: string;
  [key: string]: any;
}

export function useTelemetry() {
  const [events, setEvents] = useState<TelemetryEvent[]>([]);
  const [status, setStatus] = useState<'connecting' | 'connected' | 'disconnected'>('connecting');

  useEffect(() => {
    let ws: WebSocket;
    
    function connect() {
      ws = new WebSocket('ws://localhost:3000/ws/telemetry');
      
      ws.onopen = () => {
        setStatus('connected');
      };
      
      ws.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);
          setEvents(prev => [...prev, data]);
        } catch (e) {
          console.error('Failed to parse telemetry data', e);
        }
      };
      
      ws.onclose = () => {
        setStatus('disconnected');
      };
    }

    connect();

    return () => {
      if (ws) {
        ws.close();
      }
    };
  }, []);

  return { events, status };
}
