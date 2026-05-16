export type NodeStatus = 'Success' | 'Failure' | 'Running' | 'Idle';

export interface EditorNode {
  id: string;
  type: string;
  status: NodeStatus;
  dataPorts: Record<string, string>; // Maps local port name to global blackboard key
}

export interface Edge {
  source: string;
  sourceHandle: string;
  target: string;
  targetHandle: string;
}

export class EditorLogic {
  nodes: EditorNode[] = [];
  edges: Edge[] = [];

  constructor(nodes: EditorNode[], edges: Edge[] = []) {
    this.nodes = nodes;
    this.edges = edges;
  }

  // Feature A: Live Telemetry
  public processTelemetryEvent(nodeId: string, status: NodeStatus) {
    const node = this.nodes.find(n => n.id === nodeId);
    if (node) {
      node.status = status;
    }
  }

  // Feature B: Visual Blackboard Port Mapping
  public resolveBlackboardMappings(): Record<string, Record<string, string>> {
    const mappings: Record<string, Record<string, string>> = {};
    
    // Initialize mappings for all nodes
    for (const node of this.nodes) {
      mappings[node.id] = {};
    }

    // Connect them using a global unique key derived from the source
    for (const edge of this.edges) {
      const globalKey = `${edge.source}_${edge.sourceHandle}`;
      
      if (mappings[edge.source]) mappings[edge.source][edge.sourceHandle] = globalKey;
      if (mappings[edge.target]) mappings[edge.target][edge.targetHandle] = globalKey;
    }

    return mappings;
  }

  // Feature C: Subtrees
  public flattenSubtrees(subtrees: Record<string, EditorNode[]>): EditorNode[] {
    const flattened: EditorNode[] = [];
    
    for (const node of this.nodes) {
      if (node.type.startsWith('Subtree_')) {
        const templateNodes = subtrees[node.type];
        if (templateNodes) {
          for (const subNode of templateNodes) {
            flattened.push({
              ...subNode,
              id: `${node.id}_${subNode.id}` // Avoid ID collisions
            });
          }
        }
      } else {
        flattened.push(node);
      }
    }
    
    return flattened;
  }

  // Feature D: Dynamic Node Registry
  public loadRegistrySchema(schemaJson: string): any[] {
    const parsed = JSON.parse(schemaJson);
    return parsed.nodes || [];
  }
}
