use crate::domain::commands::RuleContextCommand;
use crate::domain::conceptual_graph::{RuleContext, RuleContextId};
use crate::domain::events::{DomainEvent, RuleContextEvent};
use crate::infrastructure::event_store::EventStore;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Handler for rule context commands
pub struct RuleContextHandler {
    event_store: Arc<dyn EventStore>,
    contexts: Arc<RwLock<HashMap<RuleContextId, RuleContext>>>,
}

impl RuleContextHandler {
    pub fn new(event_store: Arc<dyn EventStore>) -> Self {
        Self {
            event_store,
            contexts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn handle_command(&self, command: RuleContextCommand) -> Result<Vec<DomainEvent>, String> {
        match command {
            RuleContextCommand::CreateRuleContext { name, domain_context } => {
                let context_id = RuleContextId::new();
                let context = RuleContext::new(name.clone(), domain_context);

                let mut contexts = self.contexts.write().await;
                contexts.insert(context_id, context);

                let event = DomainEvent::RuleContext(RuleContextEvent::RuleContextCreated {
                    context_id,
                    name: name.clone(),
                    domain_context: domain_context.clone(),
                });

                self.event_store.append_events(context_id.to_string(), vec![event.clone()])
                    .await
                    .map_err(|e| e.to_string())?;

                Ok(vec![event])
            }

            RuleContextCommand::AddRule { context_id, rule } => {
                let mut contexts = self.contexts.write().await;
                let context = contexts.get_mut(&context_id)
                    .ok_or_else(|| "Rule context not found".to_string())?;

                context.add_rule(rule.clone())?;

                let event = DomainEvent::RuleContext(RuleContextEvent::RuleAdded {
                    context_id,
                    rule: rule.clone(),
                });

                self.event_store.append_events(context_id.to_string(), vec![event.clone()])
                    .await
                    .map_err(|e| e.to_string())?;

                Ok(vec![event])
            }

            RuleContextCommand::RemoveRule { context_id, rule_id } => {
                let mut contexts = self.contexts.write().await;
                let context = contexts.get_mut(&context_id)
                    .ok_or_else(|| "Rule context not found".to_string())?;

                context.remove_rule(rule_id)?;

                let event = DomainEvent::RuleContext(RuleContextEvent::RuleRemoved {
                    context_id,
                    rule_id,
                });

                self.event_store.append_events(context_id.to_string(), vec![event.clone()])
                    .await
                    .map_err(|e| e.to_string())?;

                Ok(vec![event])
            }

            RuleContextCommand::SetRuleEnabled { context_id, rule_id, enabled } => {
                let mut contexts = self.contexts.write().await;
                let context = contexts.get_mut(&context_id)
                    .ok_or_else(|| "Rule context not found".to_string())?;

                let rule = context.rules.get_mut(&rule_id)
                    .ok_or_else(|| "Rule not found".to_string())?;

                rule.enabled = enabled;

                let event = DomainEvent::RuleContext(RuleContextEvent::RuleEnabledChanged {
                    context_id,
                    rule_id,
                    enabled,
                });

                self.event_store.append_events(context_id.to_string(), vec![event.clone()])
                    .await
                    .map_err(|e| e.to_string())?;

                Ok(vec![event])
            }

            RuleContextCommand::EvaluateRules { context_id, concept_id, facts } => {
                let contexts = self.contexts.read().await;
                let context = contexts.get(&context_id)
                    .ok_or_else(|| "Rule context not found".to_string())?;

                let evaluation = context.evaluate(concept_id, &facts)?;

                let event = DomainEvent::RuleContext(RuleContextEvent::RulesEvaluated {
                    context_id,
                    evaluation,
                });

                self.event_store.append_events(context_id.to_string(), vec![event.clone()])
                    .await
                    .map_err(|e| e.to_string())?;

                Ok(vec![event])
            }

            RuleContextCommand::CheckCompliance { context_id, concept_id, facts } => {
                let contexts = self.contexts.read().await;
                let context = contexts.get(&context_id)
                    .ok_or_else(|| "Rule context not found".to_string())?;

                let result = context.check_compliance(concept_id, &facts);

                let event = DomainEvent::RuleContext(RuleContextEvent::ComplianceChecked {
                    context_id,
                    result: result.clone(),
                });

                self.event_store.append_events(context_id.to_string(), vec![event.clone()])
                    .await
                    .map_err(|e| e.to_string())?;

                Ok(vec![event])
            }

            RuleContextCommand::InferFacts { context_id, facts } => {
                let contexts = self.contexts.read().await;
                let context = contexts.get(&context_id)
                    .ok_or_else(|| "Rule context not found".to_string())?;

                let inferred = context.infer_facts(&facts)?;

                let event = DomainEvent::RuleContext(RuleContextEvent::FactsInferred {
                    context_id,
                    inferred_facts: inferred.clone(),
                });

                self.event_store.append_events(context_id.to_string(), vec![event.clone()])
                    .await
                    .map_err(|e| e.to_string())?;

                Ok(vec![event])
            }

            RuleContextCommand::AnalyzeImpact { context_id, fact_change } => {
                let contexts = self.contexts.read().await;
                let context = contexts.get(&context_id)
                    .ok_or_else(|| "Rule context not found".to_string())?;

                let affected_rules = context.analyze_impact(&fact_change);

                let event = DomainEvent::RuleContext(RuleContextEvent::ImpactAnalyzed {
                    context_id,
                    affected_rules,
                    fact_type: fact_change.fact_type.clone(),
                });

                self.event_store.append_events(context_id.to_string(), vec![event.clone()])
                    .await
                    .map_err(|e| e.to_string())?;

                Ok(vec![event])
            }

            RuleContextCommand::UpdateRulePriority { context_id, rule_id, new_priority } => {
                let mut contexts = self.contexts.write().await;
                let context = contexts.get_mut(&context_id)
                    .ok_or_else(|| "Rule context not found".to_string())?;

                let rule = context.rules.get_mut(&rule_id)
                    .ok_or_else(|| "Rule not found".to_string())?;

                let old_priority = rule.priority;
                rule.priority = new_priority;

                let event = RuleContextEvent::RulePriorityUpdated {
                    context_id,
                    rule_id,
                    old_priority,
                    new_priority,
                };

                Ok(vec![DomainEvent::RuleContext(event)])
            }

            RuleContextCommand::AddFact { context_id, concept_id, fact_type, value } => {
                let event = RuleContextEvent::FactAdded {
                    context_id,
                    concept_id,
                    fact_type,
                    value,
                };

                Ok(vec![DomainEvent::RuleContext(event)])
            }

            RuleContextCommand::RemoveFact { context_id, concept_id, fact_type } => {
                let event = RuleContextEvent::FactRemoved {
                    context_id,
                    concept_id,
                    fact_type,
                };

                Ok(vec![DomainEvent::RuleContext(event)])
            }

            RuleContextCommand::ExecuteRuleActions { context_id, rule_id, concept_id } => {
                let contexts = self.contexts.read().await;
                let context = contexts.get(&context_id)
                    .ok_or_else(|| "Rule context not found".to_string())?;

                let rule = context.rules.get(&rule_id)
                    .ok_or_else(|| "Rule not found".to_string())?;

                // Execute actions (simplified)
                let actions_performed: Vec<String> = rule.actions.iter()
                    .map(|action| format!("{:?}", action))
                    .collect();

                let event = RuleContextEvent::RuleActionsExecuted {
                    context_id,
                    rule_id,
                    concept_id,
                    actions_performed,
                };

                Ok(vec![DomainEvent::RuleContext(event)])
            }

            RuleContextCommand::ValidateRules { context_id } => {
                let contexts = self.contexts.read().await;
                let context = contexts.get(&context_id)
                    .ok_or_else(|| "Rule context not found".to_string())?;

                // Validate rules (simplified)
                let validation_results = HashMap::new();

                let event = RuleContextEvent::RulesValidated {
                    context_id,
                    validation_results,
                };

                Ok(vec![DomainEvent::RuleContext(event)])
            }

            RuleContextCommand::ExportRules { context_id, format } => {
                let contexts = self.contexts.read().await;
                let context = contexts.get(&context_id)
                    .ok_or_else(|| "Rule context not found".to_string())?;

                let event = DomainEvent::RuleContext(RuleContextEvent::RulesExported {
                    context_id,
                    format: format!("{:?}", format),
                    rule_count: context.rules.len(),
                });

                self.event_store.append_events(context_id.to_string(), vec![event.clone()])
                    .await
                    .map_err(|e| e.to_string())?;

                Ok(vec![event])
            }

            RuleContextCommand::ImportRules { context_id, rules_data: _, format } => {
                // Import rules (simplified)
                let imported_count = 0;
                let failed_count = 0;

                let event = RuleContextEvent::RulesImported {
                    context_id,
                    format: format!("{:?}", format),
                    imported_count,
                    failed_count,
                };

                Ok(vec![DomainEvent::RuleContext(event)])
            }
        }
    }
}
