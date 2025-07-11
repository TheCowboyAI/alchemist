-- PostgreSQL initialization script for Alchemist
-- This script sets up the database schema and initial configuration

-- Create extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_stat_statements";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";
CREATE EXTENSION IF NOT EXISTS "btree_gin";

-- Create schemas
CREATE SCHEMA IF NOT EXISTS alchemist;
CREATE SCHEMA IF NOT EXISTS alchemist_audit;

-- Set default search path
ALTER DATABASE alchemist SET search_path TO alchemist, public;

-- Create custom types
CREATE TYPE alchemist.domain_status AS ENUM ('active', 'inactive', 'maintenance');
CREATE TYPE alchemist.workflow_state AS ENUM ('pending', 'running', 'completed', 'failed', 'cancelled');
CREATE TYPE alchemist.ai_provider AS ENUM ('anthropic', 'openai', 'ollama', 'custom');

-- Create tables
-- Domains table
CREATE TABLE IF NOT EXISTS alchemist.domains (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    module_path VARCHAR(500) NOT NULL,
    status alchemist.domain_status DEFAULT 'active',
    config JSONB DEFAULT '{}',
    dependencies TEXT[] DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Workflows table
CREATE TABLE IF NOT EXISTS alchemist.workflows (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    domain_id UUID REFERENCES alchemist.domains(id),
    definition JSONB NOT NULL,
    version INTEGER DEFAULT 1,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(name, version)
);

-- Workflow executions table
CREATE TABLE IF NOT EXISTS alchemist.workflow_executions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    workflow_id UUID REFERENCES alchemist.workflows(id),
    state alchemist.workflow_state DEFAULT 'pending',
    input_data JSONB DEFAULT '{}',
    output_data JSONB DEFAULT '{}',
    error_message TEXT,
    started_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP WITH TIME ZONE,
    metadata JSONB DEFAULT '{}'
);

-- AI interactions table
CREATE TABLE IF NOT EXISTS alchemist.ai_interactions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    provider alchemist.ai_provider NOT NULL,
    model_name VARCHAR(255) NOT NULL,
    prompt TEXT NOT NULL,
    response TEXT,
    tokens_used INTEGER,
    cost_cents INTEGER,
    latency_ms INTEGER,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Dialog sessions table
CREATE TABLE IF NOT EXISTS alchemist.dialog_sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id VARCHAR(255),
    context JSONB DEFAULT '{}',
    message_count INTEGER DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    ended_at TIMESTAMP WITH TIME ZONE
);

-- Dialog messages table
CREATE TABLE IF NOT EXISTS alchemist.dialog_messages (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id UUID REFERENCES alchemist.dialog_sessions(id) ON DELETE CASCADE,
    role VARCHAR(50) NOT NULL CHECK (role IN ('user', 'assistant', 'system')),
    content TEXT NOT NULL,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Graph nodes table
CREATE TABLE IF NOT EXISTS alchemist.graph_nodes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    graph_id VARCHAR(255) NOT NULL,
    node_type VARCHAR(255) NOT NULL,
    properties JSONB DEFAULT '{}',
    position POINT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Graph edges table
CREATE TABLE IF NOT EXISTS alchemist.graph_edges (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    graph_id VARCHAR(255) NOT NULL,
    source_node_id UUID REFERENCES alchemist.graph_nodes(id) ON DELETE CASCADE,
    target_node_id UUID REFERENCES alchemist.graph_nodes(id) ON DELETE CASCADE,
    edge_type VARCHAR(255) NOT NULL,
    properties JSONB DEFAULT '{}',
    weight DECIMAL(10, 4) DEFAULT 1.0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Policies table
CREATE TABLE IF NOT EXISTS alchemist.policies (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    rules JSONB NOT NULL,
    is_active BOOLEAN DEFAULT true,
    priority INTEGER DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Audit log table
CREATE TABLE IF NOT EXISTS alchemist_audit.audit_log (
    id BIGSERIAL PRIMARY KEY,
    table_name VARCHAR(255) NOT NULL,
    operation VARCHAR(10) NOT NULL,
    user_id VARCHAR(255),
    row_id UUID,
    old_data JSONB,
    new_data JSONB,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes
CREATE INDEX idx_domains_status ON alchemist.domains(status);
CREATE INDEX idx_workflows_domain_id ON alchemist.workflows(domain_id);
CREATE INDEX idx_workflows_active ON alchemist.workflows(is_active);
CREATE INDEX idx_workflow_executions_workflow_id ON alchemist.workflow_executions(workflow_id);
CREATE INDEX idx_workflow_executions_state ON alchemist.workflow_executions(state);
CREATE INDEX idx_workflow_executions_started_at ON alchemist.workflow_executions(started_at DESC);
CREATE INDEX idx_ai_interactions_provider ON alchemist.ai_interactions(provider);
CREATE INDEX idx_ai_interactions_created_at ON alchemist.ai_interactions(created_at DESC);
CREATE INDEX idx_dialog_sessions_user_id ON alchemist.dialog_sessions(user_id);
CREATE INDEX idx_dialog_messages_session_id ON alchemist.dialog_messages(session_id);
CREATE INDEX idx_graph_nodes_graph_id ON alchemist.graph_nodes(graph_id);
CREATE INDEX idx_graph_edges_graph_id ON alchemist.graph_edges(graph_id);
CREATE INDEX idx_graph_edges_source_target ON alchemist.graph_edges(source_node_id, target_node_id);
CREATE INDEX idx_policies_active ON alchemist.policies(is_active);
CREATE INDEX idx_audit_log_table_operation ON alchemist_audit.audit_log(table_name, operation);
CREATE INDEX idx_audit_log_created_at ON alchemist_audit.audit_log(created_at DESC);

-- GIN indexes for JSONB columns
CREATE INDEX idx_domains_config_gin ON alchemist.domains USING gin(config);
CREATE INDEX idx_workflows_definition_gin ON alchemist.workflows USING gin(definition);
CREATE INDEX idx_workflow_executions_metadata_gin ON alchemist.workflow_executions USING gin(metadata);
CREATE INDEX idx_graph_nodes_properties_gin ON alchemist.graph_nodes USING gin(properties);
CREATE INDEX idx_policies_rules_gin ON alchemist.policies USING gin(rules);

-- Create update timestamp trigger function
CREATE OR REPLACE FUNCTION alchemist.update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Apply update timestamp triggers
CREATE TRIGGER update_domains_updated_at BEFORE UPDATE ON alchemist.domains
    FOR EACH ROW EXECUTE FUNCTION alchemist.update_updated_at_column();

CREATE TRIGGER update_workflows_updated_at BEFORE UPDATE ON alchemist.workflows
    FOR EACH ROW EXECUTE FUNCTION alchemist.update_updated_at_column();

CREATE TRIGGER update_dialog_sessions_updated_at BEFORE UPDATE ON alchemist.dialog_sessions
    FOR EACH ROW EXECUTE FUNCTION alchemist.update_updated_at_column();

CREATE TRIGGER update_graph_nodes_updated_at BEFORE UPDATE ON alchemist.graph_nodes
    FOR EACH ROW EXECUTE FUNCTION alchemist.update_updated_at_column();

CREATE TRIGGER update_policies_updated_at BEFORE UPDATE ON alchemist.policies
    FOR EACH ROW EXECUTE FUNCTION alchemist.update_updated_at_column();

-- Create audit trigger function
CREATE OR REPLACE FUNCTION alchemist_audit.audit_trigger_function()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        INSERT INTO alchemist_audit.audit_log(table_name, operation, row_id, new_data)
        VALUES (TG_TABLE_NAME, TG_OP, NEW.id, to_jsonb(NEW));
        RETURN NEW;
    ELSIF TG_OP = 'UPDATE' THEN
        INSERT INTO alchemist_audit.audit_log(table_name, operation, row_id, old_data, new_data)
        VALUES (TG_TABLE_NAME, TG_OP, NEW.id, to_jsonb(OLD), to_jsonb(NEW));
        RETURN NEW;
    ELSIF TG_OP = 'DELETE' THEN
        INSERT INTO alchemist_audit.audit_log(table_name, operation, row_id, old_data)
        VALUES (TG_TABLE_NAME, TG_OP, OLD.id, to_jsonb(OLD));
        RETURN OLD;
    END IF;
    RETURN NULL;
END;
$$ language 'plpgsql';

-- Apply audit triggers to important tables
CREATE TRIGGER audit_domains AFTER INSERT OR UPDATE OR DELETE ON alchemist.domains
    FOR EACH ROW EXECUTE FUNCTION alchemist_audit.audit_trigger_function();

CREATE TRIGGER audit_workflows AFTER INSERT OR UPDATE OR DELETE ON alchemist.workflows
    FOR EACH ROW EXECUTE FUNCTION alchemist_audit.audit_trigger_function();

CREATE TRIGGER audit_policies AFTER INSERT OR UPDATE OR DELETE ON alchemist.policies
    FOR EACH ROW EXECUTE FUNCTION alchemist_audit.audit_trigger_function();

-- Create read-only user for reporting
CREATE ROLE alchemist_readonly;
GRANT CONNECT ON DATABASE alchemist TO alchemist_readonly;
GRANT USAGE ON SCHEMA alchemist TO alchemist_readonly;
GRANT SELECT ON ALL TABLES IN SCHEMA alchemist TO alchemist_readonly;
ALTER DEFAULT PRIVILEGES IN SCHEMA alchemist GRANT SELECT ON TABLES TO alchemist_readonly;

-- Insert default data
INSERT INTO alchemist.domains (name, description, module_path, status) VALUES
    ('graph', 'Core graph operations and spatial positioning', 'cim-domain-graph', 'active'),
    ('workflow', 'Business process execution and state machines', 'cim-domain-workflow', 'active'),
    ('agent', 'AI provider integration and semantic search', 'cim-domain-agent', 'active'),
    ('document', 'Document lifecycle and version control', 'cim-domain-document', 'active'),
    ('policy', 'Business rule enforcement', 'cim-domain-policy', 'active')
ON CONFLICT (name) DO NOTHING;

-- Performance settings
ALTER DATABASE alchemist SET random_page_cost = 1.1;
ALTER DATABASE alchemist SET effective_cache_size = '3GB';
ALTER DATABASE alchemist SET shared_buffers = '1GB';
ALTER DATABASE alchemist SET work_mem = '16MB';
ALTER DATABASE alchemist SET maintenance_work_mem = '256MB';

-- Grant permissions
GRANT ALL PRIVILEGES ON SCHEMA alchemist TO alchemist;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA alchemist TO alchemist;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA alchemist TO alchemist;
GRANT ALL PRIVILEGES ON SCHEMA alchemist_audit TO alchemist;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA alchemist_audit TO alchemist;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA alchemist_audit TO alchemist;