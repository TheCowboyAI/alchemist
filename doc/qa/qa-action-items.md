# QA Action Items Summary

## Immediate Actions Required

### 🔴 High Priority
1. **Start Phase 1: Edge Visualization**
   - Implement RenderGraphEdges service
   - Create EdgeVisual components
   - Test with current 3-node example

### 🟡 Medium Priority
1. **Add Automated Tests**
   - Implement fitness functions from QA report
   - Add integration tests for event flows
   - Create CI/CD naming convention checks

2. **Documentation Updates**
   - Keep vocabulary.md current with new terms
   - Add code examples to design documents
   - Create developer onboarding guide

### 🟢 Low Priority
1. **Technical Debt**
   - Consider renaming GraphEvent enum to StoredEvent
   - This is internal to repository layer

## Compliance Maintenance

### Daily Checklist
- [ ] Events are past-tense without suffix
- [ ] Services are verb phrases
- [ ] Storage uses plural terms
- [ ] No technical suffixes (Manager, Handler, etc.)

### Code Review Focus
1. Check against vocabulary.md
2. Verify bounded context isolation
3. Ensure event-driven patterns
4. Validate domain language

## Current Status Summary

### What's Working Well (100% Compliant)
- ✅ DDD naming conventions
- ✅ Bevy ECS patterns
- ✅ NixOS environment
- ✅ Rust standards
- ✅ Event-driven architecture

### What Needs Work (70% Complete)
- ⚠️ Edge visualization (Phase 1)
- ⚠️ Selection system (Phase 2)
- ⚠️ Daggy integration (Phase 3)
- ⚠️ Layout algorithms (Phase 4)
- ⚠️ Import/Export formats (Phase 5)

## Next Sprint Goals
1. Complete Phase 1: Edge Visualization
2. Start Phase 2: Selection System
3. Maintain 100% DDD compliance
4. Add basic integration tests

---

*Generated from*: [DDD Compliance Quality Assurance Report](./ddd-compliance-quality-assurance-report.md)
