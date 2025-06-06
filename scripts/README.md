# CIM Scripts

This directory contains utility scripts for the CIM project.

## Vocabulary Generation

### generate-vocabulary-md.py

This script demonstrates how `vocabulary.md` is a **projection** of the canonical vocabulary data stored in `vocabulary-graph.json`.

#### Projection Concept

Following our event-sourcing principles:
- **Source of Truth**: `vocabulary-graph.json` contains the complete, structured vocabulary data
- **Projection**: `vocabulary.md` is a read-optimized view generated from the JSON
- **Regeneration**: The markdown can be regenerated anytime from the JSON source

#### Usage

```bash
./scripts/generate-vocabulary-md.py
```

This will:
1. Read `doc/publish/vocabulary-graph.json`
2. Generate `doc/publish/vocabulary.md` based on the projection rules
3. Report the number of terms and categories processed

#### Benefits

1. **Single Source of Truth**: All vocabulary data lives in one structured format
2. **Multiple Projections**: Can generate different views (markdown, HTML, graph visualizations)
3. **Consistency**: Ensures vocabulary.md always matches the canonical data
4. **Extensibility**: Easy to add new fields or relationships to the JSON

#### JSON Structure

The vocabulary graph JSON contains:
- **metadata**: Version, creation date, projection rules
- **categories**: Hierarchical organization of terms
- **terms**: Individual vocabulary entries with:
  - Unique ID
  - Name and definition
  - Category and subcategory
  - Type and taxonomy
  - Relationships to other terms
  - Usage context
  - Code references
- **relationships**: Explicit edges between terms

This structure allows us to:
- Generate consistent documentation
- Visualize term relationships as a graph
- Query vocabulary programmatically
- Track vocabulary evolution over time
