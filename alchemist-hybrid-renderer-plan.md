# Alchemist: Hybrid Bevy+Iced Rendering Architecture

## Core Concept

Alchemist uses **multiple rendering engines** to display domain objects in the most intuitive way:
- **Bevy**: 3D graphs, spatial relationships, workflows
- **Iced**: Documents, UI controls, text, media
- **Unified**: Both renderers work together in the same application

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    Alchemist Application                     │
├─────────────────────────────────────────────────────────────┤
│                    Domain Object Layer                       │
│  Git Repo │ Flake.nix │ Progress.json │ Documents │ Music   │
├─────────────────────────────────────────────────────────────┤
│                    Render Router                             │
│         "What's the best way to display this?"              │
├──────────────────────┬──────────────────────────────────────┤
│    Bevy Renderer     │         Iced Renderer                │
│  • 3D Graphs         │    • Document viewer                 │
│  • Workflows         │    • Code editor                     │
│  • Spatial data      │    • Progress dashboard              │
│  • Organizations     │    • Media player                    │
│  • Relationships     │    • Forms & controls                │
└──────────────────────┴──────────────────────────────────────┘
```

## Technical Implementation

### 1. Unified Window Management

```rust
// alchemist/src/main.rs
pub struct AlchemistApp {
    // The main window contains both renderers
    window: winit::window::Window,
    
    // Bevy runs in a viewport within the window
    bevy_app: bevy::app::App,
    bevy_viewport: Viewport,
    
    // Iced handles the main UI chrome
    iced_app: iced::Application,
    
    // Domain objects are shared
    domain_registry: Arc<RwLock<DomainRegistry>>,
    
    // Render router decides which renderer to use
    render_router: RenderRouter,
}

impl AlchemistApp {
    pub fn new() -> Self {
        // Create a single window
        let window = create_window();
        
        // Initialize Bevy in a specific viewport
        let bevy_app = create_bevy_app(&window);
        
        // Initialize Iced for the main UI
        let iced_app = create_iced_app(&window);
        
        // Share domain data between renderers
        let domain_registry = Arc::new(RwLock::new(DomainRegistry::new()));
        
        Self {
            window,
            bevy_app,
            bevy_viewport: Viewport::default(),
            iced_app,
            domain_registry,
            render_router: RenderRouter::new(),
        }
    }
}
```

### 2. Render Router

```rust
pub struct RenderRouter {
    rules: Vec<RenderRule>,
}

pub struct RenderRule {
    domain_type: DomainType,
    view_preference: ViewPreference,
    renderer: RendererType,
}

impl RenderRouter {
    pub fn route(&self, object: &DomainObject, user_pref: Option<ViewPreference>) -> RendererType {
        match (object.domain_type(), user_pref) {
            // Documents always render in Iced unless user wants 3D
            (DomainType::Document, Some(ViewPreference::Spatial)) => RendererType::Bevy,
            (DomainType::Document, _) => RendererType::Iced,
            
            // Graphs and workflows default to Bevy
            (DomainType::Graph, _) => RendererType::Bevy,
            (DomainType::Workflow, _) => RendererType::Bevy,
            
            // Media uses Iced
            (DomainType::Music, _) => RendererType::Iced,
            (DomainType::Video, _) => RendererType::Iced,
            
            // Code and config files use Iced
            (DomainType::Code, _) => RendererType::Iced,
            (DomainType::Config, _) => RendererType::Iced,
            
            // Organizations can be either
            (DomainType::Organization, Some(ViewPreference::Tree)) => RendererType::Iced,
            (DomainType::Organization, _) => RendererType::Bevy,
            
            _ => RendererType::Iced, // Default to 2D
        }
    }
}
```

### 3. Hybrid UI Layout

```rust
// The main window is divided into regions
pub struct AlchemistLayout {
    // Iced handles the outer chrome
    header: IcedHeader,         // File menu, toolbar
    sidebar: IcedSidebar,       // Domain browser, properties
    footer: IcedFooter,         // Status bar, console
    
    // Center can be either Bevy or Iced
    center: CenterView,
    
    // Floating panels can overlay
    panels: Vec<FloatingPanel>,
}

pub enum CenterView {
    Bevy(BevyViewport),        // 3D visualization
    Iced(IcedDocument),        // Document/media view
    Split(Box<CenterView>, Box<CenterView>), // Split view
}

pub struct FloatingPanel {
    content: PanelContent,
    position: Position,
    size: Size,
    renderer: RendererType,
}
```

### 4. Domain Object Rendering

#### Reading Git Repository
```rust
impl GitDomainRenderer {
    pub fn render_repository(&self, repo_path: &Path) -> RenderOutput {
        // Parse the repository structure
        let repo = GitRepository::open(repo_path)?;
        
        // Create graph representation
        let graph = self.create_commit_graph(&repo);
        
        // Render in Bevy as 3D graph
        RenderOutput::Bevy(BevyScene {
            nodes: graph.commits.map(|c| Node3D {
                id: c.id,
                position: layout_3d(&c),
                visual: NodeVisual::Commit(c),
            }),
            edges: graph.parent_relationships,
        })
    }
}
```

#### Reading flake.nix
```rust
impl NixDomainRenderer {
    pub fn render_flake(&self, flake_path: &Path) -> RenderOutput {
        // Parse flake.nix
        let flake = NixFlake::parse(flake_path)?;
        
        // User can choose view
        match self.view_preference {
            ViewPreference::Graph => {
                // Bevy: Show dependency graph
                RenderOutput::Bevy(self.create_dependency_graph(&flake))
            }
            ViewPreference::Code => {
                // Iced: Syntax-highlighted editor
                RenderOutput::Iced(IcedWidget::CodeEditor {
                    content: flake.source,
                    language: Language::Nix,
                })
            }
        }
    }
}
```

#### Reading progress.json
```rust
impl ProgressRenderer {
    pub fn render_progress(&self, progress_path: &Path) -> RenderOutput {
        let progress = Progress::load(progress_path)?;
        
        // Multiple views available
        match self.view_preference {
            ViewPreference::Dashboard => {
                // Iced: Clean dashboard with charts
                RenderOutput::Iced(self.create_dashboard(&progress))
            }
            ViewPreference::Graph => {
                // Bevy: 3D milestone network
                RenderOutput::Bevy(self.create_milestone_graph(&progress))
            }
            ViewPreference::Timeline => {
                // Iced: Gantt chart
                RenderOutput::Iced(self.create_timeline(&progress))
            }
        }
    }
}
```

### 5. Unified Event System

```rust
// Events flow between renderers
pub enum AlchemistEvent {
    // User interactions
    IcedEvent(iced::Event),
    BevyEvent(bevy::input::Input),
    
    // Domain events
    DomainEvent(DomainEvent),
    
    // Render events
    SwitchRenderer {
        object: DomainObject,
        target: RendererType,
    },
    OpenInNewWindow {
        object: DomainObject,
        renderer: RendererType,
    },
}

impl AlchemistApp {
    pub fn handle_event(&mut self, event: AlchemistEvent) {
        match event {
            AlchemistEvent::IcedEvent(e) => {
                // Iced handles its events
                self.iced_app.update(e);
            }
            AlchemistEvent::BevyEvent(e) => {
                // Bevy handles its events
                self.bevy_app.update();
            }
            AlchemistEvent::DomainEvent(e) => {
                // Update domain objects
                self.domain_registry.write().unwrap().apply_event(e);
                // Refresh both renderers
                self.refresh_views();
            }
            AlchemistEvent::SwitchRenderer { object, target } => {
                self.switch_renderer(object, target);
            }
            _ => {}
        }
    }
}
```

### 6. Example: Complete UI Flow

```
┌─────────────────────────────────────────────────────────────┐
│ Alchemist - [project: alchemist] [branch: main]    [-][□][X]│
├─────────────────────────────────────────────────────────────┤
│ File  View  Domain  Analyze  Create  Help                   │
├─────┬───────────────────────────────────────────────────────┤
│     │                    (Bevy Viewport)                     │
│  D  │                                                        │
│  o  │         ┌────────┐     ┌────────┐                     │
│  m  │         │Progress│────>│Domain  │                     │
│  a  │         │ 100%   │     │Models  │                     │
│  i  │         └────────┘     └────────┘                     │
│  n  │              │              │                          │
│  s  │              v              v                          │
│     │         ┌────────┐     ┌────────┐                     │
│  •  │         │Workflow│     │  Git   │                     │
│  G  │         │ Engine │<────│ Repos  │                     │
│  i  │         └────────┘     └────────┘                     │
│  t  │                                                        │
│  •  ├────────────────────────────────────────────────────────┤
│  D  │ (Iced Panel) progress.json                    [3D][2D]│
│  o  │ ┌────────────────────────────────────────────────────┐│
│  c  │ │{                                                    ││
│  s  │ │  "project": "CIM",                                  ││
│  •  │ │  "completion": 100,                                 ││
│  W  │ │  "domains": {...}                                   ││
│  o  │ └────────────────────────────────────────────────────┘│
│  r  ├────────────────────────────────────────────────────────┤
│  k  │ Status: Connected to NATS | Events: 1,234/s | FPS: 60 │
└─────┴───────────────────────────────────────────────────────┘
```

### 7. Implementation Steps

#### Phase 1: Basic Integration (2 weeks)
1. Create unified window with winit
2. Embed Bevy in a viewport
3. Wrap window with Iced
4. Share event loop
5. Test basic rendering

#### Phase 2: Domain Readers (3 weeks)
1. Git repository reader → Bevy graph
2. flake.nix reader → Iced editor
3. progress.json reader → Both renderers
4. Document reader → Iced viewer
5. Music/video players → Iced

#### Phase 3: Render Router (1 week)
1. Define routing rules
2. User preference system
3. Dynamic switching
4. Performance optimization

#### Phase 4: Unified UX (2 weeks)
1. Consistent theme
2. Smooth transitions
3. Drag & drop between renderers
4. Keyboard shortcuts
5. Context menus

### 8. Key Benefits

1. **Best Tool for Each Job**: 3D for graphs, 2D for documents
2. **User Choice**: Switch views based on preference
3. **Performance**: Only render what's needed
4. **Familiarity**: Documents look like documents, not 3D objects
5. **Innovation**: Unique hybrid approach

### 9. Technical Challenges & Solutions

| Challenge | Solution |
|-----------|----------|
| Event loop conflict | Single winit event loop, dispatch to both |
| Rendering performance | Viewport clipping, lazy rendering |
| State synchronization | Shared domain registry with Arc<RwLock<>> |
| Visual consistency | Shared theme system, coordinated colors |
| Input handling | Unified input router, context-aware dispatch |

## Conclusion

Alchemist's hybrid rendering approach gives us the best of both worlds:
- **Bevy** for stunning 3D visualizations of relationships and flows
- **Iced** for crisp, familiar 2D interfaces for documents and media
- **Unified** experience that feels cohesive and intuitive

The key insight is that different types of information are best understood through different visual metaphors. By choosing the right renderer for each domain object, we create an interface that's both powerful and intuitive.