# pib.OS Editor (Frontend)

This directory contains the visual drag-and-drop Behavior Tree editor for pib.OS.

## Architecture: Micro-Frontend (Web Component)
The editor is built using **React, TypeScript, and Vite** to leverage the powerful ecosystem of node-based editing tools (like React Flow). However, because the main robotics dashboard (**pib.Cerebra**) is built in **Angular**, this project is designed as a Micro-Frontend.

It serves two purposes:
1. **Standalone Application:** It runs as a standalone React app for rapid UI development and Test-Driven Development (TDD).
2. **Web Component Export:** The production build packages the entire React application into a framework-agnostic HTML Custom Element (`<pib-os-editor>`). This allows the Angular team to seamlessly drop the editor into `pib.Cerebra` without dealing with React dependencies.

## Standalone Execution & Development

You can run the editor completely independently of `pib.Cerebra` to test nodes, styling, and the WebSocket connection.

### Installation
```bash
cd ui
npm install
```

### Run the Dev Server (Standalone)
This will spin up a local Vite development server where you can interact with the editor in your browser.
```bash
npm run dev
```

### Run the Tests (TDD)
We enforce a strict RED-GREEN-REFACTOR workflow using **Vitest**. All tree logic and JSON serialization must be tested here:
```bash
npm run test
```

## Integration into pib.Cerebra (Angular)
*(Note: Web Component build script configuration is upcoming)*
Once built via `npm run build`, the output can be imported into Angular. You can then use it natively in any Angular template:

```html
<!-- Inside pib.Cerebra Angular Template -->
<pib-os-editor 
    [treeData]="currentRobotTree" 
    (onExport)="handleTreeExport($event)">
</pib-os-editor>
```
