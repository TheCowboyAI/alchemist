# RSS Feed Processing with Real NLP Integration

This document describes how the Alchemist RSS feed processor integrates with real AI providers for NLP analysis.

## Overview

The RSS feed processor fetches RSS feeds, enriches them with NLP analysis using real AI providers (OpenAI, Anthropic, Ollama), and publishes the enriched content to NATS for consumption by the dashboard and other services.

## Features

### NLP Enrichments
- **Sentiment Analysis**: Determines positive, negative, or neutral sentiment with confidence scores
- **Entity Extraction**: Identifies people, organizations, locations, technologies, products, and events
- **Keyword Extraction**: Extracts the most important keywords and phrases
- **Summarization**: Generates concise AI-powered summaries
- **Relevance Scoring**: Calculates relevance based on NLP analysis quality

### Supported AI Providers
- OpenAI (GPT-3.5, GPT-4)
- Anthropic (Claude)
- Ollama (Local models)

## Architecture

```
┌─────────────────┐     ┌──────────────┐     ┌─────────────┐
│   RSS Feeds     │────▶│ RSS Processor│────▶│ AI Provider │
│                 │     │              │     │  (NLP API)  │
└─────────────────┘     └──────┬───────┘     └─────────────┘
                               │
                               ▼
                        ┌──────────────┐
                        │ NATS JetStream│
                        │   (Events)    │
                        └──────┬───────┘
                               │
                               ▼
                        ┌──────────────┐
                        │  Dashboard   │
                        │   (Iced UI)  │
                        └──────────────┘
```

## Configuration

### 1. Configure AI Provider

First, add an AI model to Alchemist:

```bash
# Add OpenAI
alchemist ai add openai-gpt4 openai
export OPENAI_API_KEY="your-api-key"

# Add Anthropic
alchemist ai add claude-3 anthropic
export ANTHROPIC_API_KEY="your-api-key"

# Add Ollama (local)
alchemist ai add llama2 ollama --endpoint http://localhost:11434

# Test the model
alchemist ai test openai-gpt4
```

### 2. Start NATS Server

The RSS processor requires NATS for event streaming:

```bash
# Using Docker
docker run -p 4222:4222 -p 8222:8222 nats:latest -js

# Or using nats-server
nats-server -js
```

### 3. Run RSS Processor Service

Start the standalone RSS processor service:

```bash
# Run with default settings
rss-processor

# Run with custom AI model
rss-processor --ai-model claude-3

# Run with debug logging
rss-processor --debug

# Specify NATS URL
rss-processor --nats-url nats://localhost:4222
```

### 4. Monitor Events

The processor publishes events to NATS subjects:

- `rss.processed.items` - Enriched RSS items with NLP analysis
- `rss.feeds.*.status` - Feed processing status updates
- `rss.feeds.*.errors` - Processing errors
- `rss.feeds.*.filtered` - Filtered items

Monitor using NATS CLI:
```bash
# Subscribe to all RSS events
nats sub "rss.>"

# Subscribe to processed items only
nats sub "rss.processed.items"
```

## Feed Configuration

Feeds are configured with filters and transformations:

```rust
RssFeedConfig {
    id: "tech-news",
    name: "Tech News Aggregator",
    url: "https://news.ycombinator.com/rss",
    category: "technology",
    update_interval: 300, // 5 minutes
    filters: vec![
        FeedFilter {
            filter_type: FilterType::TitleContains,
            value: "AI",
        },
    ],
    transformations: vec![
        FeedTransformation {
            transform_type: TransformType::ExtractEntities,
            config: json!({}),
        },
        FeedTransformation {
            transform_type: TransformType::SentimentAnalysis,
            config: json!({}),
        },
        FeedTransformation {
            transform_type: TransformType::ExtractKeywords,
            config: json!({}),
        },
    ],
    enabled: true,
}
```

## NLP Processing Details

### Sentiment Analysis
- Score: -1.0 (negative) to 1.0 (positive)
- Label: positive, negative, or neutral
- Confidence: 0.0 to 1.0

### Entity Extraction
Types detected:
- PERSON - People's names
- ORGANIZATION - Companies, institutions
- LOCATION - Places, addresses
- TECHNOLOGY - Technologies, frameworks
- PRODUCT - Product names
- EVENT - Events, conferences

### Keyword Extraction
- Extracts up to 5 most important keywords
- Uses AI to identify key concepts

### Summarization
- Generates concise summaries (max 200 characters)
- Preserves key information

## Example Enriched RSS Item

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "feed_id": "tech-news",
  "feed_name": "Tech News Aggregator",
  "title": "OpenAI Announces GPT-5 with Enhanced Reasoning",
  "description": "OpenAI has unveiled GPT-5...",
  "link": "https://example.com/article",
  "pub_date": "2024-01-15T10:30:00Z",
  "author": "John Doe",
  "categories": ["AI", "Technology"],
  "sentiment": {
    "score": 0.85,
    "label": "positive",
    "confidence": 0.92
  },
  "entities": [
    {
      "text": "OpenAI",
      "entity_type": "ORGANIZATION",
      "confidence": 0.98
    },
    {
      "text": "GPT-5",
      "entity_type": "TECHNOLOGY",
      "confidence": 0.95
    }
  ],
  "keywords": [
    "artificial intelligence",
    "reasoning",
    "language model",
    "breakthrough",
    "OpenAI"
  ],
  "summary": "OpenAI unveils GPT-5 with significantly enhanced reasoning capabilities, marking a major breakthrough in AI language models.",
  "relevance_score": 0.89,
  "processed_at": "2024-01-15T10:31:00Z"
}
```

## Dashboard Integration

The Alchemist dashboard displays RSS feeds with NLP enrichments:

1. **RSS Feeds Tab**: Shows all configured feeds and their status
2. **Recent Items**: Displays enriched RSS items with sentiment indicators
3. **Statistics**: Shows sentiment distribution, top entities, and keywords
4. **Real-time Updates**: Updates as new items are processed

Start the dashboard:
```bash
alchemist dashboard
```

## Performance Considerations

### Rate Limiting
- The NLP processor respects AI provider rate limits
- Configurable batch sizes for efficient processing
- Automatic retry with exponential backoff

### Caching
- Processed GUIDs are cached to avoid reprocessing
- NLP results can be cached (future enhancement)

### Resource Usage
- Each feed is processed in a separate task
- Configurable update intervals per feed
- Memory-efficient streaming processing

## Troubleshooting

### Common Issues

1. **AI Model Not Available**
   - Check API keys are set
   - Verify model name is correct
   - Test with `alchemist ai test <model>`

2. **NATS Connection Failed**
   - Ensure NATS server is running
   - Check NATS URL is correct
   - Verify network connectivity

3. **NLP Processing Errors**
   - Check AI provider quotas
   - Verify API key permissions
   - Enable debug logging for details

### Debug Mode

Run with debug logging to see detailed processing:
```bash
rss-processor --debug
```

### Health Checks

Monitor service health:
- Check NATS connection status
- Monitor AI provider response times
- Track processing success/error rates

## Future Enhancements

- [ ] Configurable NLP analysis per feed
- [ ] Custom prompt templates
- [ ] Result caching and deduplication
- [ ] Batch processing optimization
- [ ] Multi-language support
- [ ] Custom entity types
- [ ] Trend analysis over time
- [ ] Alert rules based on NLP results