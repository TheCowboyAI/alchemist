# Workflow Domain Missing Methods

## Overview
This document tracks all the missing methods that need to be implemented in the Workflow aggregate to make the user story tests pass.

## Missing Methods in Workflow Aggregate

### Progress and Monitoring Methods
1. `get_progress() -> WorkflowProgress`
   - Returns overall workflow progress statistics
   - Includes: total_steps, completed_steps, in_progress_steps, pending_steps, failed_steps, percentage_complete

2. `get_step_details() -> Vec<StepDetail>`
   - Returns detailed information about all steps
   - Includes: name, status, started_at, completed_at, assigned_to, etc.

3. `get_bottlenecks(threshold: Duration) -> Vec<StepDetail>`
   - Identifies steps taking longer than threshold
   - Returns steps that are potential bottlenecks

4. `get_critical_path() -> Vec<StepDetail>`
   - Calculates the longest dependency chain
   - Returns the critical path through the workflow

5. `get_timeout_risks() -> Vec<StepDetail>`
   - Identifies steps with timeout configurations
   - Returns steps that might timeout

### Task Assignment Methods
6. `get_assignable_tasks() -> Vec<&WorkflowStep>`
   - Returns tasks that can be assigned (unassigned manual/approval steps)

7. `assign_task(step_id: StepId, assignee: String, assigned_by: Option<String>) -> DomainResult<Vec<WorkflowDomainEvent>>`
   - Assigns a task to a specific user
   - Generates TaskAssigned event

8. `get_tasks_for_assignee(assignee: &str) -> Vec<&WorkflowStep>`
   - Returns all tasks assigned to a specific user

9. `reassign_task(step_id: StepId, new_assignee: String, reassigned_by: Option<String>) -> DomainResult<Vec<WorkflowDomainEvent>>`
   - Reassigns a task from one user to another
   - Generates TaskReassigned event

10. `get_high_priority_tasks() -> Vec<&WorkflowStep>`
    - Returns tasks marked as high priority

### Task Completion Methods
11. `complete_task(step_id: StepId, completed_by: String, completion_data: HashMap<String, Value>) -> DomainResult<Vec<WorkflowDomainEvent>>`
    - Completes a human task with form data
    - Generates TaskCompleted event

12. `update_task_progress(step_id: StepId, progress: u8, updated_by: String) -> DomainResult<Vec<WorkflowDomainEvent>>`
    - Updates task progress percentage
    - Generates TaskProgressUpdated event

### Integration Methods
13. `register_integration(step_id: StepId, integration_id: String, config: IntegrationConfig) -> DomainResult<Vec<WorkflowDomainEvent>>`
    - Registers an external system integration
    - Generates IntegrationRegistered event

14. `record_integration_call(step_id: StepId, status: IntegrationStatus, response: Option<Value>) -> DomainResult<Vec<WorkflowDomainEvent>>`
    - Records integration call result
    - Generates IntegrationCalled event

### Error Handling Methods
15. `retry_step(step_id: StepId, retry_by: Option<String>) -> DomainResult<Vec<WorkflowDomainEvent>>`
    - Retries a failed step
    - Generates StepRetried event

16. `compensate_step(step_id: StepId, compensation_data: HashMap<String, Value>) -> DomainResult<Vec<WorkflowDomainEvent>>`
    - Executes compensation for a step
    - Generates StepCompensated event

17. `rollback(reason: String, rollback_by: Option<String>) -> DomainResult<Vec<WorkflowDomainEvent>>`
    - Initiates workflow rollback
    - Generates WorkflowRolledBack event

### Circuit Breaker Methods
18. `open_circuit_breaker(step_id: StepId, reason: String) -> DomainResult<Vec<WorkflowDomainEvent>>`
    - Opens circuit breaker for a step
    - Generates CircuitBreakerOpened event

19. `close_circuit_breaker(step_id: StepId) -> DomainResult<Vec<WorkflowDomainEvent>>`
    - Closes circuit breaker for a step
    - Generates CircuitBreakerClosed event

20. `check_circuit_breaker(step_id: StepId) -> CircuitBreakerState`
    - Returns current circuit breaker state

### SLA and Performance Methods
21. `check_sla_violations() -> Vec<SLAViolation>`
    - Checks for SLA violations across all steps

22. `get_performance_metrics() -> WorkflowPerformanceMetrics`
    - Returns performance metrics for the workflow

23. `get_step_performance(step_id: StepId) -> Option<StepPerformanceMetrics>`
    - Returns performance metrics for a specific step

### Parallel Execution Methods
24. `split_parallel(step_id: StepId, branches: Vec<ParallelBranch>) -> DomainResult<Vec<WorkflowDomainEvent>>`
    - Creates parallel execution branches
    - Generates ParallelSplitCreated event

25. `join_parallel(join_point_id: StepId, completed_branch: StepId) -> DomainResult<Vec<WorkflowDomainEvent>>`
    - Joins parallel branches
    - Generates ParallelBranchJoined event

### Loop Pattern Methods
26. `create_loop(step_id: StepId, condition: LoopCondition, max_iterations: u32) -> DomainResult<Vec<WorkflowDomainEvent>>`
    - Creates a loop pattern
    - Generates LoopCreated event

27. `evaluate_loop_condition(loop_id: StepId) -> bool`
    - Evaluates if loop should continue

### Scheduling Methods
28. `schedule_execution(schedule: WorkflowSchedule) -> DomainResult<Vec<WorkflowDomainEvent>>`
    - Schedules workflow execution
    - Generates WorkflowScheduled event

29. `update_schedule(new_schedule: WorkflowSchedule) -> DomainResult<Vec<WorkflowDomainEvent>>`
    - Updates workflow schedule
    - Generates ScheduleUpdated event

### Sub-workflow Methods
30. `create_sub_workflow(parent_step_id: StepId, sub_workflow_id: WorkflowId) -> DomainResult<Vec<WorkflowDomainEvent>>`
    - Creates a sub-workflow relationship
    - Generates SubWorkflowCreated event

31. `complete_sub_workflow(parent_step_id: StepId, sub_workflow_id: WorkflowId, results: HashMap<String, Value>) -> DomainResult<Vec<WorkflowDomainEvent>>`
    - Completes a sub-workflow
    - Generates SubWorkflowCompleted event

### Versioning Methods
32. `create_version(version_tag: String, created_by: String) -> DomainResult<Vec<WorkflowDomainEvent>>`
    - Creates a new workflow version
    - Generates WorkflowVersionCreated event

33. `migrate_instance(to_version: String) -> DomainResult<Vec<WorkflowDomainEvent>>`
    - Migrates running instance to new version
    - Generates InstanceMigrated event

### Transaction Methods
34. `begin_transaction(transaction_id: TransactionId) -> DomainResult<Vec<WorkflowDomainEvent>>`
    - Begins a workflow transaction
    - Generates TransactionBegan event

35. `commit_transaction(transaction_id: TransactionId) -> DomainResult<Vec<WorkflowDomainEvent>>`
    - Commits a workflow transaction
    - Generates TransactionCommitted event

36. `rollback_transaction(transaction_id: TransactionId, reason: String) -> DomainResult<Vec<WorkflowDomainEvent>>`
    - Rolls back a workflow transaction
    - Generates TransactionRolledBack event

## Missing Methods in WorkflowStep

1. `start(assigned_to: Option<String>) -> DomainResult<()>`
   - Starts step execution
   - Updates status to InProgress

2. `complete() -> DomainResult<()>`
   - Completes step execution
   - Updates status to Completed

3. `fail(error: String) -> DomainResult<()>`
   - Marks step as failed
   - Updates status to Failed

4. `is_completed() -> bool`
   - Returns true if step is completed

5. `can_execute(completed_steps: &[StepId]) -> bool`
   - Checks if all dependencies are satisfied

## Missing Event Types

The following event variants need to be added to `WorkflowDomainEvent`:

1. `TaskAssigned`
2. `TaskReassigned`
3. `TaskCompleted`
4. `TaskProgressUpdated`
5. `IntegrationRegistered`
6. `IntegrationCalled`
7. `StepRetried`
8. `StepCompensated`
9. `WorkflowRolledBack`
10. `CircuitBreakerOpened`
11. `CircuitBreakerClosed`
12. `ParallelSplitCreated`
13. `ParallelBranchJoined`
14. `LoopCreated`
15. `WorkflowScheduled`
16. `ScheduleUpdated`
17. `SubWorkflowCreated`
18. `SubWorkflowCompleted`
19. `WorkflowVersionCreated`
20. `InstanceMigrated`
21. `TransactionBegan`
22. `TransactionCommitted`
23. `TransactionRolledBack`

## Missing Value Objects

1. `WorkflowProgress` - Progress tracking structure
2. `StepDetail` - Detailed step information
3. `SLAViolation` - SLA violation details
4. `WorkflowPerformanceMetrics` - Performance metrics
5. `StepPerformanceMetrics` - Step-specific metrics
6. `IntegrationConfig` - Integration configuration
7. `IntegrationStatus` - Integration call status
8. `CircuitBreakerState` - Circuit breaker state
9. `ParallelBranch` - Parallel branch configuration
10. `LoopCondition` - Loop condition specification
11. `WorkflowSchedule` - Scheduling configuration
12. `TransactionId` - Transaction identifier

## Missing StepStatus Variant

1. `InProgress` - Step is currently being executed

## Implementation Priority

### Phase 1: Core Functionality (Required for basic tests)
- Progress tracking methods
- Task assignment methods
- Basic step lifecycle methods
- Core events

### Phase 2: Advanced Features
- Integration methods
- Error handling and compensation
- Circuit breakers
- Performance tracking

### Phase 3: Workflow Patterns
- Parallel execution
- Loop patterns
- Sub-workflows

### Phase 4: Enterprise Features
- Scheduling
- Versioning
- Transactions

## Next Steps

1. Implement missing methods in phases
2. Add missing event types to domain events
3. Create missing value objects
4. Update tests as methods are implemented
5. Document each method with proper rustdoc comments 