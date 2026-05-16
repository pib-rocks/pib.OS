export type NodeType = 'Sequence' | 'Selector' | 'Parallel' | 'Action' | 'Condition';

export interface BTNode {
  id: string;
  type: NodeType;
  children?: BTNode[];
  params?: Record<string, any>;
}

export function exportTreeToJson(root: BTNode | null): string {
  // GREEN PHASE: Check for null and serialize properly
  if (!root) {
    throw new Error('Cannot export an empty Behavior Tree');
  }
  
  // Pretty-print JSON for readability in the export file
  return JSON.stringify(root, null, 2);
}
