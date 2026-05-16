import { describe, it, expect } from 'vitest';
import { exportTreeToJson, BTNode } from './treeExporter';

describe('Tree Exporter (TDD)', () => {
  it('must serialize a Sequence with two Actions to a strictly typed JSON string', () => {
    const tree: BTNode = {
      id: 'root-seq',
      type: 'Sequence',
      children: [
        { id: 'act-1', type: 'Action', params: { command: 'move' } },
        { id: 'act-2', type: 'Action', params: { command: 'grip' } }
      ]
    };

    const jsonString = exportTreeToJson(tree);
    
    // Attempt to parse it back to prove it's valid JSON
    const parsed = JSON.parse(jsonString);
    
    expect(parsed.type).toBe('Sequence');
    expect(parsed.children.length).toBe(2);
    expect(parsed.children[0].params.command).toBe('move');
  });

  it('must throw a validation error given an empty tree', () => {
    expect(() => exportTreeToJson(null)).toThrow('Cannot export an empty Behavior Tree');
  });
});
