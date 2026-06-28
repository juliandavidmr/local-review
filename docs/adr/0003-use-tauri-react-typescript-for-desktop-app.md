# Use Tauri, React, and TypeScript for the Desktop App

The application will use Tauri with a React and TypeScript frontend as its initial desktop technology stack. This fits a local-first review tool that needs a modern interface plus controlled access to the filesystem, Git, `gh`, local model providers, and MCP integrations without the runtime weight of Electron.

**Considered Options**

- Electron with React and TypeScript.
- A browser-based local web app.
- Native platform-specific desktop apps.
- Tauri with React and TypeScript.

**Consequences**

The product can keep a web-style UI development model while using Tauri commands for local process and filesystem integration. The core desktop boundary will need careful design so review logic remains testable and not tightly coupled to UI components.
