import { useState, useEffect } from 'react';

export interface NodeRegistryItem {
  id: string;
  [key: string]: any;
}

export function useNodeRegistry() {
  const [nodes, setNodes] = useState<NodeRegistryItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let isMounted = true;
    
    async function fetchNodes() {
      try {
        const response = await fetch('http://localhost:3000/api/registry');
        if (!response.ok) {
          throw new Error('Failed to fetch node registry');
        }
        const data = await response.json();
        if (isMounted) {
          setNodes(data);
          setError(null);
        }
      } catch (err: any) {
        if (isMounted) {
          setError(err.message || 'Failed to fetch node registry');
        }
      } finally {
        if (isMounted) {
          setLoading(false);
        }
      }
    }

    fetchNodes();

    return () => {
      isMounted = false;
    };
  }, []);

  return { nodes, loading, error };
}
