import { useState, useEffect } from 'react'
import './App.css'

function App() {
  const [nodes, setNodes] = useState<any[]>([]);

  useEffect(() => {
    fetch('/api/registry')
      .then(res => res.json())
      .then(data => setNodes(data))
      .catch(() => {});
  }, []);

  return (
    <div>
      <div className="node-toolbox">
        <h2>Node Toolbox</h2>
        {nodes.map((node, i) => (
          <div key={i}>{node.name}</div>
        ))}
      </div>
      <div className="react-flow" style={{ width: '500px', height: '500px', border: '1px solid black' }}>
        Canvas/Editor
      </div>
    </div>
  )
}

export default App
