import { useState, useEffect } from 'react'
import './App.css'

function App() {
  const [nodes, setNodes] = useState<any[]>([]);
  const [selectedNode, setSelectedNode] = useState<any>(null);
  const [config, setConfig] = useState<string>('{}');

  useEffect(() => {
    fetch('/api/registry')
      .then(res => res.json())
      .then(data => setNodes(data))
      .catch(() => {});
  }, []);

  const handleNodeClick = (node: any) => {
    setSelectedNode(node);
    if (node.config_schema) {
      setConfig(JSON.stringify({ schema_placeholder: "replace with valid schema values" }, null, 2));
    } else {
      setConfig('{}');
    }
  };

  return (
    <div style={{ display: 'flex', gap: '20px' }}>
      <div className="node-toolbox">
        <h2>Node Toolbox</h2>
        {nodes.map((node, i) => (
          <div 
            key={i} 
            onClick={() => handleNodeClick(node)}
            style={{ cursor: 'pointer', padding: '5px', border: '1px solid #ccc', marginBottom: '5px' }}
          >
            {node.name}
          </div>
        ))}
      </div>
      <div className="react-flow" style={{ width: '500px', height: '500px', border: '1px solid black' }}>
        Canvas/Editor
      </div>
      
      {/* Properties Panel */}
      {selectedNode && (
        <div className="properties-panel" style={{ width: '300px', border: '1px solid blue', padding: '10px' }}>
          <h3>Properties: {selectedNode.name}</h3>
          {selectedNode.config_schema ? (
            <div>
              <label>Config (JSON):</label>
              <textarea 
                value={config} 
                onChange={(e) => setConfig(e.target.value)}
                style={{ width: '100%', height: '200px' }}
              />
            </div>
          ) : (
            <p>No configuration schema for this node.</p>
          )}
        </div>
      )}
    </div>
  )
}

export default App
