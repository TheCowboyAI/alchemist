# CIM Graph Editor Tutorial

Welcome to the Composable Information Machine (CIM) Graph Editor! This tutorial will guide you through creating, editing, and managing visual workflows and knowledge graphs.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Basic Navigation](#basic-navigation)
3. [Creating Your First Graph](#creating-your-first-graph)
4. [Working with Nodes](#working-with-nodes)
5. [Connecting Nodes](#connecting-nodes)
6. [Graph Types and Use Cases](#graph-types-and-use-cases)
7. [Advanced Features](#advanced-features)
8. [Keyboard Shortcuts](#keyboard-shortcuts)
9. [Best Practices](#best-practices)

## Getting Started

### Launching the Graph Editor

1. **Start CIM**:
   ```bash
   nix run
   ```

2. **Open your browser** to `http://localhost:8080`

3. **Login** with your credentials

4. **Click "New Graph"** or select an existing graph from the dashboard

### Interface Overview

```
┌─────────────────────────────────────────────────────────────┐
│  File  Edit  View  Graph  Tools  Help                      │
├─────────────────────────────────────────────────────────────┤
│ [New] [Open] [Save] | [Undo] [Redo] | [Zoom] [Pan] [Reset] │
├──────────────┬──────────────────────────────────────────────┤
│              │                                               │
│   Node       │                                               │
│   Palette    │            3D Graph Canvas                    │
│              │                                               │
│   - Workflow │                                               │
│   - Concept  │                                               │
│   - Event    │                                               │
│   - Custom   │                                               │
│              │                                               │
├──────────────┴───────────────────────────────────────────────┤
│ Properties Panel | Minimap | Console                         │
└──────────────────────────────────────────────────────────────┘
```

## Basic Navigation

### Camera Controls

- **Rotate**: Left-click and drag
- **Pan**: Right-click and drag (or Middle-click)
- **Zoom**: Scroll wheel
- **Focus Node**: Double-click on a node
- **Reset View**: Press `R` or click Reset button

### Selection

- **Select Node**: Left-click
- **Multi-select**: Ctrl+click or drag selection box
- **Select All**: Ctrl+A
- **Deselect**: Click empty space or press Escape

## Creating Your First Graph

### Step 1: Choose Graph Type

Click "New Graph" and select a type:

- **Workflow Graph**: For business processes
- **Conceptual Graph**: For knowledge representation
- **Event Flow Graph**: For system events
- **Development Graph**: For project tracking

### Step 2: Add Your First Node

1. **Drag from palette** or right-click canvas
2. Choose node type:
   - Start Node (workflows)
   - Process Node
   - Decision Node
   - End Node

3. **Name your node** in the properties panel

### Step 3: Configure Node Properties

Select a node to edit its properties:

```yaml
Node Properties:
  Name: "Approve Document"
  Type: "Approval"
  Description: "Manager approval required"
  Timeout: "24 hours"
  Assignee: "manager-role"
  Metadata:
    priority: "high"
    department: "finance"
```

## Working with Nodes

### Node Types

#### Workflow Nodes

- **Start**: Entry point (green)
- **Process**: Action step (blue)
- **Decision**: Conditional branch (yellow)
- **Parallel**: Fork/join (purple)
- **End**: Completion (red)

#### Conceptual Nodes

- **Concept**: Core idea (circle)
- **Category**: Grouping (rectangle)
- **Instance**: Specific example (diamond)

### Node Operations

- **Move**: Drag node to new position
- **Resize**: Drag corner handles (some types)
- **Copy**: Ctrl+C, then Ctrl+V
- **Delete**: Select and press Delete
- **Group**: Select multiple, right-click → Group

### Quick Node Creation

1. **Tab Menu**: Press Tab for quick add
2. **Type to filter**: Start typing node name
3. **Enter to create**: At cursor position

## Connecting Nodes

### Creating Edges

1. **Connection Mode**: Hold Shift or click Connect tool
2. **Click source node** (connection point appears)
3. **Click target node** to complete connection

### Edge Types

- **Sequence** (→): Default flow
- **Conditional** (⇢): With condition
- **Parallel** (⇉): Simultaneous execution
- **Data Flow** (⤍): Information transfer

### Configuring Edges

Select an edge to configure:

```yaml
Edge Properties:
  Type: "Conditional"
  Label: "If approved"
  Condition: "status == 'approved'"
  Priority: 1
  Style:
    color: "#00ff00"
    thickness: 2
    animated: true
```

## Graph Types and Use Cases

### Workflow Graphs

Perfect for business processes:

```
[Start] → [Submit Request] → <Manager Approval?>
                                    ↓ Yes        ↓ No
                            [Process Order]  [Notify Rejection]
                                    ↓              ↓
                                 [End]          [End]
```

**Example Use Cases**:
- Document approval workflows
- Order processing
- Employee onboarding
- Customer service flows

### Conceptual Graphs

Ideal for knowledge representation:

```
    [Machine Learning]
         /    |    \
   [Neural] [Trees] [SVM]
      |        |      |
  [Deep Learning] [Random Forest]
```

**Example Use Cases**:
- Knowledge bases
- Ontologies
- Concept maps
- Semantic networks

### Event Flow Graphs

Visualize system events:

```
[User Login] → [Auth Event] → [Session Created]
                    ↓
              [Log Event] → [Analytics]
```

**Example Use Cases**:
- System architecture
- Event sourcing flows
- Microservice communication
- Debugging event chains

## Advanced Features

### Subgraphs

Create hierarchical structures:

1. **Select nodes** to group
2. **Right-click** → "Create Subgraph"
3. **Name** the subgraph
4. **Double-click** to enter/exit

### Templates

Save and reuse graph patterns:

1. **Select elements** to save as template
2. **Graph** → "Save as Template"
3. **Name** and categorize
4. **Drag** from template palette to use

### Semantic Layout

Let AI optimize your graph layout:

1. **Graph** → "Semantic Layout"
2. Choose optimization:
   - Minimize crossings
   - Cluster similar nodes
   - Hierarchical arrangement
   - Force-directed layout

### Real-time Collaboration

Work with others simultaneously:

- **See cursors** of other users
- **Live updates** as changes happen
- **Chat** in sidebar
- **Comments** on nodes

### Version Control

Track graph changes:

1. **Save versions** with Ctrl+S
2. **View history** in Timeline panel
3. **Compare versions** side-by-side
4. **Restore** previous versions

## Keyboard Shortcuts

### Essential Shortcuts

| Action     | Windows/Linux | Mac         |
| ---------- | ------------- | ----------- |
| New Graph  | Ctrl+N        | Cmd+N       |
| Save       | Ctrl+S        | Cmd+S       |
| Undo       | Ctrl+Z        | Cmd+Z       |
| Redo       | Ctrl+Y        | Cmd+Shift+Z |
| Delete     | Delete        | Delete      |
| Copy       | Ctrl+C        | Cmd+C       |
| Paste      | Ctrl+V        | Cmd+V       |
| Select All | Ctrl+A        | Cmd+A       |

### Navigation Shortcuts

| Action        | Key              |
| ------------- | ---------------- |
| Pan Mode      | Space+Drag       |
| Zoom In       | + or Ctrl+Scroll |
| Zoom Out      | - or Ctrl+Scroll |
| Fit to Screen | F                |
| Reset View    | R                |
| Focus Search  | /                |

### Node Shortcuts

| Action       | Key          |
| ------------ | ------------ |
| Quick Add    | Tab          |
| Connect Mode | Shift+Drag   |
| Multi-select | Ctrl+Click   |
| Duplicate    | Ctrl+D       |
| Group        | Ctrl+G       |
| Ungroup      | Ctrl+Shift+G |

## Best Practices

### Graph Organization

1. **Use consistent layouts**:
   - Top-to-bottom for workflows
   - Radial for concepts
   - Left-to-right for timelines

2. **Group related nodes**:
   - By function
   - By department
   - By phase

3. **Use meaningful names**:
   - Descriptive node labels
   - Clear edge conditions
   - Helpful descriptions

### Visual Clarity

1. **Minimize edge crossings**
2. **Align nodes** to grid (Ctrl+Shift+A)
3. **Use colors** meaningfully:
   - Green = Start/Success
   - Red = End/Error
   - Yellow = Decision
   - Blue = Process

4. **Add annotations** for complex areas

### Performance Tips

1. **Use subgraphs** for large graphs (>100 nodes)
2. **Enable LOD** (Level of Detail) for better performance
3. **Hide unnecessary details** when zoomed out
4. **Use filters** to focus on specific aspects

### Collaboration Guidelines

1. **Lock nodes** you're actively editing
2. **Use comments** instead of chat for persistent info
3. **Name your versions** descriptively
4. **Coordinate** major structural changes

## Workflow Examples

### Document Approval Workflow

```
1. Create "Start" node
2. Add "Upload Document" process node
3. Add "Review Document" process node
4. Add "Approval Decision" decision node
5. Create two paths:
   - Approved → "Publish" → End
   - Rejected → "Request Changes" → loop back
6. Set timeouts and assignees
7. Test with sample data
```

### Knowledge Graph Creation

```
1. Start with central concept
2. Add related concepts as nodes
3. Connect with relationship types:
   - "is-a" (hierarchy)
   - "part-of" (composition)
   - "related-to" (association)
4. Use semantic layout to optimize
5. Add metadata for AI processing
```

## Troubleshooting

### Common Issues

**Nodes not connecting**:
- Ensure compatible node types
- Check connection rules in properties
- Verify no circular dependencies

**Performance issues**:
- Reduce visible nodes with filters
- Enable performance mode (F9)
- Use subgraphs for organization

**Collaboration conflicts**:
- Refresh to sync (F5)
- Check version history
- Coordinate with team members

## Next Steps

1. **Explore templates** in the Template Gallery
2. **Join a workshop** on advanced features
3. **Read the API docs** for automation
4. **Share your graphs** with the community

## Getting Help

- **In-app help**: Press F1
- **Video tutorials**: Help → Video Guides
- **Community forum**: https://forum.cim.dev
- **Support**: support@cim.dev

Happy graph editing! 🎨📊 