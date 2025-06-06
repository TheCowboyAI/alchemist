#!/usr/bin/env python3
"""
Generate vocabulary.md from vocabulary-graph.json

This script demonstrates how vocabulary.md is a projection of the canonical
vocabulary data stored in JSON format.
"""

import json
import sys
from pathlib import Path
from typing import Dict, List, Any


def load_vocabulary_graph(path: Path) -> Dict[str, Any]:
    """Load the vocabulary graph JSON file."""
    with open(path, 'r') as f:
        return json.load(f)


def format_term_name(term: Dict[str, Any]) -> str:
    """Format the term name for markdown."""
    name = term.get('name', term.get('id', ''))
    # Remove parenthetical suffixes like "(Component)" if they exist
    if ' (' in name and name.endswith(')'):
        name = name[:name.rfind(' (')]
    return name


def format_relationships(relationships: Dict[str, List[str]]) -> str:
    """Format relationships as a bullet list."""
    if not relationships:
        return ""

    lines = []
    for rel_type, targets in relationships.items():
        rel_name = rel_type.replace('-', ' ').title()
        targets_str = ', '.join(targets)
        lines.append(f"  * {rel_name}: {targets_str}")

    return '\n'.join(lines)


def generate_term_section(term: Dict[str, Any], level: str = "####") -> str:
    """Generate markdown for a single term."""
    lines = [
        f"{level} Term: {format_term_name(term)}",
        f"- **Category**: {term.get('type', 'Unknown')}",
        f"- **Type**: {term.get('type', 'Unknown')}",
    ]

    if term.get('taxonomy'):
        lines.append(f"- **Taxonomy**: {term['taxonomy']}")

    lines.extend([
        f"- **Definition**: {term.get('definition', 'No definition provided')}",
        "- **Relationships**:",
        format_relationships(term.get('relationships', {})),
        f"- **Usage Context**: {term.get('usage_context', 'Not specified')}",
        f"- **Code Reference**: `{term.get('code_reference', 'TBD')}`" if term.get('code_reference') else "- **Code Reference**: TBD"
    ])

    return '\n'.join(lines)


def generate_markdown(vocab_data: Dict[str, Any]) -> str:
    """Generate the complete vocabulary.md content."""
    lines = [
        "# CIM Vocabulary",
        "",
        "[← Back to Index](index.md)",
        ""
    ]

    # Group terms by category and subcategory
    terms_by_category = {}
    for term in vocab_data['terms']:
        cat_id = term['category']
        subcat_id = term.get('subcategory')

        if cat_id not in terms_by_category:
            terms_by_category[cat_id] = {'subcategories': {}}

        if subcat_id:
            if subcat_id not in terms_by_category[cat_id]['subcategories']:
                terms_by_category[cat_id]['subcategories'][subcat_id] = []
            terms_by_category[cat_id]['subcategories'][subcat_id].append(term)
        else:
            if 'direct' not in terms_by_category[cat_id]:
                terms_by_category[cat_id]['direct'] = []
            terms_by_category[cat_id]['direct'].append(term)

    # Generate sections based on categories
    for category in vocab_data['categories']:
        cat_id = category['id']
        if cat_id not in terms_by_category:
            continue

        lines.extend([
            f"## {category['name']}",
            ""
        ])

        if category.get('description'):
            lines.extend([
                f"*{category['description']}*",
                ""
            ])

        # Handle subcategories
        if category.get('subcategories'):
            for subcat in category['subcategories']:
                subcat_id = subcat['id']
                if subcat_id in terms_by_category[cat_id]['subcategories']:
                    lines.extend([
                        f"### {subcat['name']}",
                        ""
                    ])

                    if subcat.get('description'):
                        lines.extend([
                            f"*{subcat['description']}*",
                            ""
                        ])

                    # Add terms in this subcategory
                    for term in terms_by_category[cat_id]['subcategories'][subcat_id]:
                        lines.extend([
                            generate_term_section(term),
                            ""
                        ])

        # Handle direct terms (no subcategory)
        if 'direct' in terms_by_category[cat_id]:
            for term in terms_by_category[cat_id]['direct']:
                lines.extend([
                    generate_term_section(term, "###"),
                    ""
                ])

    # Add footer
    lines.extend([
        "---",
        "",
        "*This vocabulary is continuously updated as the system evolves. For the latest implementation details, refer to the source code and documentation.*"
    ])

    return '\n'.join(lines)


def main():
    """Main entry point."""
    # Determine paths
    script_dir = Path(__file__).parent
    project_root = script_dir.parent

    vocab_json_path = project_root / 'doc' / 'publish' / 'vocabulary-graph.json'
    vocab_md_path = project_root / 'doc' / 'publish' / 'vocabulary.md'

    # Load vocabulary data
    try:
        vocab_data = load_vocabulary_graph(vocab_json_path)
    except FileNotFoundError:
        print(f"Error: Could not find {vocab_json_path}")
        sys.exit(1)
    except json.JSONDecodeError as e:
        print(f"Error: Invalid JSON in vocabulary file: {e}")
        sys.exit(1)

    # Generate markdown
    markdown_content = generate_markdown(vocab_data)

    # Write output
    with open(vocab_md_path, 'w') as f:
        f.write(markdown_content)

    print(f"Successfully generated {vocab_md_path} from {vocab_json_path}")
    print(f"Processed {len(vocab_data['terms'])} terms across {len(vocab_data['categories'])} categories")


if __name__ == '__main__':
    main()
