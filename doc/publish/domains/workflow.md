# Workflow Domain

## Overview

The Workflow Domain enables visual business process automation through graph-based workflow definitions. It provides a powerful engine for designing, executing, and monitoring complex business processes with support for conditional logic, parallel execution, and human-in-the-loop interactions.

## Key Concepts

### Workflow
- **Definition**: A directed graph representing a business process
- **Components**: Steps, transitions, conditions, data flow
- **Types**: Sequential, parallel, conditional, iterative
- **States**: Draft → Published → Running → Completed/Failed

### Step
- **Definition**: An atomic unit of work in a workflow
- **Types**: Start, End, Task, Decision, Fork, Join, Human Task
- **Properties**: ID, type, handler, timeout, retry policy
- **Execution**: Synchronous or asynchronous

### Transition
- **Definition**: Connection between steps with optional conditions
- **Properties**: Source step, target step, condition expression
- **Types**: Sequential, conditional, parallel, loop-back

### Execution Context
- **Definition**: Runtime state of a workflow instance
- **Contains**: Variables, current step, history, error state
- **Persistence**: Event-sourced for full auditability

## Domain Events

### Commands
- `cmd.workflow.create_workflow` - Define new workflow
- `cmd.workflow.start_execution` - Begin workflow instance
- `cmd.workflow.complete_step` - Mark step as done
- `cmd.workflow.retry_step` - Retry failed step
- `cmd.workflow.cancel_execution` - Stop workflow

### Events
- `event.workflow.workflow_created` - New workflow defined
- `event.workflow.execution_started` - Instance begun
- `event.workflow.step_completed` - Step finished
- `event.workflow.decision_made` - Branch selected
- `event.workflow.execution_completed` - Workflow done

### Queries
- `query.workflow.get_definition` - Retrieve workflow design
- `query.workflow.get_executions` - List running instances
- `query.workflow.get_execution_state` - Current status
- `query.workflow.get_step_history` - Execution timeline

## API Reference

### WorkflowAggregate
```rust
pub struct WorkflowAggregate {
    pub id: WorkflowId,
    pub name: String,
    pub version: Version,
    pub steps: HashMap<StepId, WorkflowStep>,
    pub transitions: Vec<Transition>,
    pub variables: HashMap<String, VariableDefinition>,
    pub status: WorkflowStatus,
}
```

### Key Methods
- `create_workflow()` - Define new process
- `add_step()` - Add workflow step
- `connect_steps()` - Create transition
- `validate_workflow()` - Check completeness
- `publish_workflow()` - Make available for execution

## Workflow Patterns

### Sequential Process
```rust
// Simple approval workflow
let workflow = WorkflowBuilder::new("Document Approval")
    .add_step(Step::start("Submit"))
    .add_step(Step::task("Review", ReviewHandler))
    .add_step(Step::decision("Approve?"))
    .add_step(Step::task("Notify", EmailHandler))
    .add_step(Step::end("Complete"))
    .connect("Submit", "Review")
    .connect("Review", "Approve?")
    .connect_conditional("Approve?", "Notify", "approved == true")
    .connect_conditional("Approve?", "Submit", "approved == false")
    .connect("Notify", "Complete")
    .build();
```

### Parallel Execution
```rust
// Parallel processing workflow
let workflow = WorkflowBuilder::new("Order Fulfillment")
    .add_step(Step::start("Order Received"))
    .add_step(Step::fork("Process Order"))
    .add_step(Step::task("Reserve Inventory", InventoryHandler))
    .add_step(Step::task("Process Payment", PaymentHandler))
    .add_step(Step::task("Schedule Shipping", ShippingHandler))
    .add_step(Step::join("Await All"))
    .add_step(Step::end("Order Complete"))
    .connect("Order Received", "Process Order")
    .connect_parallel("Process Order", vec![
        "Reserve Inventory",
        "Process Payment",
        "Schedule Shipping"
    ])
    .connect_join(vec![
        "Reserve Inventory",
        "Process Payment",
        "Schedule Shipping"
    ], "Await All")
    .connect("Await All", "Order Complete")
    .build();
```

### Human-in-the-Loop
```rust
// Human task workflow
let workflow = WorkflowBuilder::new("Expense Approval")
    .add_step(Step::start("Submit Expense"))
    .add_step(Step::human_task("Manager Approval", ApprovalForm))
    .add_step(Step::decision("Amount Check"))
    .add_step(Step::human_task("Director Approval", ApprovalForm))
    .add_step(Step::task("Process Reimbursement", FinanceHandler))
    .add_step(Step::end("Complete"))
    .connect("Submit Expense", "Manager Approval")
    .connect("Manager Approval", "Amount Check")
    .connect_conditional("Amount Check", "Director Approval", "amount > 5000")
    .connect_conditional("Amount Check", "Process Reimbursement", "amount <= 5000")
    .connect("Director Approval", "Process Reimbursement")
    .connect("Process Reimbursement", "Complete")
    .build();
```

## Execution Engine

### Step Handlers
```rust
pub trait StepHandler: Send + Sync {
    async fn execute(
        &self,
        context: &mut ExecutionContext,
        input: StepInput,
    ) -> Result<StepOutput, WorkflowError>;
}

// Example handler
struct EmailHandler;

impl StepHandler for EmailHandler {
    async fn execute(
        &self,
        context: &mut ExecutionContext,
        input: StepInput,
    ) -> Result<StepOutput, WorkflowError> {
        let recipient = context.get_variable("recipient")?;
        let subject = context.get_variable("subject")?;
        
        send_email(recipient, subject).await?;
        
        Ok(StepOutput::success()
            .with_variable("email_sent", true))
    }
}
```

### Error Handling
- **Retry Policies**: Exponential backoff, max attempts
- **Compensation**: Rollback actions for failures
- **Error Transitions**: Alternative paths on failure
- **Timeout Handling**: Configurable step timeouts

## Integration Features

### External System Integration
- REST API calls
- NATS message publishing
- Database operations
- File system access
- Third-party service calls

### Event Integration
```rust
// Workflow triggered by domain event
on_event("order.created", |event| {
    WorkflowExecution::start(
        "OrderFulfillment",
        Variables::from_event(event),
    )
});

// Workflow publishes domain events
workflow.on_complete(|execution| {
    publish_event("workflow.order.fulfilled", execution.result())
});
```

## Monitoring and Analytics

### Execution Metrics
- Average completion time
- Step duration analysis
- Bottleneck identification
- Success/failure rates
- SLA compliance

### Real-time Monitoring
- Active execution tracking
- Step status visualization
- Variable inspection
- Error diagnostics
- Performance profiling

## Use Cases

### Business Process Automation
- Order processing
- Document approval
- Employee onboarding
- Invoice processing
- Customer service workflows

### Integration Orchestration
- Multi-system coordination
- API workflow chains
- Data pipeline management
- Microservice orchestration

### Human Workflow Management
- Task assignment
- Approval chains
- Collaborative processes
- Escalation handling

## Performance Characteristics

- **Throughput**: 10,000+ executions/second
- **Latency**: <10ms step transitions
- **Scalability**: Horizontal scaling support
- **Persistence**: Event-sourced for reliability

## Best Practices

1. **Idempotent Steps**: Ensure steps can be safely retried
2. **Error Boundaries**: Define clear failure handling
3. **Variable Scoping**: Minimize context variables
4. **Versioning**: Support multiple workflow versions
5. **Testing**: Comprehensive step handler testing

## Related Domains

- **Graph Domain**: Visual workflow representation
- **Identity Domain**: User task assignment
- **Policy Domain**: Business rule integration
- **Document Domain**: Document-centric workflows 