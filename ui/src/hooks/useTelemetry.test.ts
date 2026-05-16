import { renderHook, waitFor, act } from '@testing-library/react';
import { useTelemetry } from './useTelemetry';
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

class MockWebSocket {
  onopen: (() => void) | null = null;
  onmessage: ((event: any) => void) | null = null;
  onclose: (() => void) | null = null;
  close = vi.fn();
  
  constructor(public url: string) {}
}

describe('useTelemetry', () => {
  let originalWebSocket: any;
  let mockWs: MockWebSocket;

  beforeEach(() => {
    originalWebSocket = global.WebSocket;
    (global as any).WebSocket = function(url: string) {
      mockWs = new MockWebSocket(url);
      return mockWs;
    };
  });

  afterEach(() => {
    (global as any).WebSocket = originalWebSocket;
  });

  it('should connect and receive telemetry events', async () => {
    const { result } = renderHook(() => useTelemetry());

    expect(result.current.status).toBe('connecting');

    act(() => {
      if (mockWs.onopen) mockWs.onopen();
    });

    expect(result.current.status).toBe('connected');

    act(() => {
      if (mockWs.onmessage) mockWs.onmessage({ data: JSON.stringify({ node_id: 'seq_1', status: 'Running' }) });
    });

    expect(result.current.events).toHaveLength(1);
    expect(result.current.events[0]).toEqual({ node_id: 'seq_1', status: 'Running' });
  });

  it('should handle disconnect', () => {
    const { result } = renderHook(() => useTelemetry());

    act(() => {
      if (mockWs.onclose) mockWs.onclose();
    });

    expect(result.current.status).toBe('disconnected');
  });
});
