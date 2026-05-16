import { describe, it, expect } from 'vitest';
import { EditorLogic, EditorNode } from './features';

describe('Groot2-like Features in React Flow (TDD)', () => {
  
  it('Feature A (PR-1232): updates node status based on telemetry', () => {
    const logic = new EditorLogic([{ id: 'n1', type: 'Action', status: 'Idle', dataPorts: {} }]);
    logic.processTelemetryEvent('n1', 'Running');
    expect(logic.nodes[0].status).toBe('Running');
  });

  it('Feature B (PR-1233): resolves visual edges into blackboard port mappings', () => {
    const nodes: EditorNode[] = [
      { id: 'cam', type: 'Camera', status: 'Idle', dataPorts: {} },
      { id: 'motor', type: 'Move', status: 'Idle', dataPorts: {} }
    ];
    // Camera's output port 'target_x' is visually connected to Motor's input port 'x'
    const edges = [{ source: 'cam', sourceHandle: 'target_x', target: 'motor', targetHandle: 'x' }];
    
    const logic = new EditorLogic(nodes, edges);
    const mapping = logic.resolveBlackboardMappings();
    
    // It should automatically create a global blackboard key like "cam_target_x"
    // and map it so the Rust ScopedBlackboard can link them.
    expect(mapping['motor']['x']).toBe('cam_target_x');
    expect(mapping['cam']['target_x']).toBe('cam_target_x');
  });

  it('Feature C (PR-1234): flattens subtrees into a single execution array', () => {
    // A tree with a node of type "Subtree_MoveToCharge"
    const nodes: EditorNode[] = [
      { id: 'n1', type: 'Subtree_MoveToCharge', status: 'Idle', dataPorts: {} }
    ];
    const subtrees = {
      'Subtree_MoveToCharge': [
        { id: 'sub1', type: 'Action_Align', status: 'Idle', dataPorts: {} },
        { id: 'sub2', type: 'Action_Dock', status: 'Idle', dataPorts: {} }
      ]
    };
    
    const logic = new EditorLogic(nodes);
    const flattened = logic.flattenSubtrees(subtrees);
    
    expect(flattened.length).toBe(2);
    expect(flattened[0].id).toBe('n1_sub1'); // Must prefix ID to avoid collisions
    expect(flattened[1].type).toBe('Action_Dock');
  });

  it('Feature D (PR-1235): loads dynamic registry schema into toolbox templates', () => {
    const logic = new EditorLogic([]);
    const schema = JSON.stringify({
      nodes: [
        { type: 'Hardware_Lidar', description: 'Reads distance', ports: ['distance'] }
      ]
    });
    
    const registry = logic.loadRegistrySchema(schema);
    expect(registry.length).toBe(1);
    expect(registry[0].type).toBe('Hardware_Lidar');
  });

});
