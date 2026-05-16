import { renderHook, waitFor } from '@testing-library/react';
import { useNodeRegistry } from './useNodeRegistry';
import { describe, it, expect, vi, beforeEach } from 'vitest';

const mockNodes = [{ id: '1', name: 'Node 1' }];

describe('useNodeRegistry', () => {
  beforeEach(() => {
    global.fetch = vi.fn();
  });

  it('should fetch nodes successfully', async () => {
    (global.fetch as any).mockResolvedValue({
      ok: true,
      json: async () => mockNodes,
    });

    const { result } = renderHook(() => useNodeRegistry());

    expect(result.current.loading).toBe(true);
    expect(result.current.nodes).toEqual([]);

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.nodes).toEqual(mockNodes);
    expect(result.current.error).toBeNull();
    expect(global.fetch).toHaveBeenCalledWith('http://localhost:3000/api/registry');
  });

  it('should handle fetch error', async () => {
    (global.fetch as any).mockResolvedValue({
      ok: false,
    });

    const { result } = renderHook(() => useNodeRegistry());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.error).toBe('Failed to fetch node registry');
    expect(result.current.nodes).toEqual([]);
  });
});
