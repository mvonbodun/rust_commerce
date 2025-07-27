# Phase 4 Subplan: MongoDB Testcontainers Integration

## Overview
This subplan addresses the testcontainers integration challenges encountered during Phase 4 implementation. The goal is to enable true integration testing with MongoDB containers for the GetBestOfferPrice API.

## Current Issue Analysis
During the initial Phase 4 implementation, we encountered:
- `testcontainers::images::mongo` module not found (private module)
- Container lifetime management issues
- API compatibility problems with testcontainers 0.15.0

## 🎯 Objective
Successfully implement MongoDB testcontainers integration to enable:
- Automated MongoDB container startup/shutdown for tests
- True integration testing against real MongoDB instances
- Isolated test environments for each test run
- Proper test data cleanup and isolation

## 📋 Detailed Subplan Steps

### Step 4.1: Research Current Testcontainers MongoDB Support
**Goal**: Understand the current state of MongoDB support in testcontainers-rs

#### Tasks:
- [ ] Research testcontainers-rs documentation for MongoDB support
- [ ] Check available MongoDB image implementations in current version
- [ ] Identify the correct way to create custom MongoDB containers
- [ ] Document the proper API usage patterns

#### Success Criteria:
- Clear understanding of how to create MongoDB containers with testcontainers 0.15
- Documented API patterns for container lifecycle management

### Step 4.2: Implement Basic MongoDB Container Setup
**Goal**: Create a working MongoDB container that starts and stops successfully

#### Tasks:
- [ ] Create a simple custom MongoDB image implementation
- [ ] Implement proper container startup with health checks
- [ ] Test basic container lifecycle (start/stop)
- [ ] Verify MongoDB connectivity to the container

#### Files to Create/Modify:
- `/price/tests/testcontainers_setup.rs` - Basic container setup
- Update `/price/Cargo.toml` with correct testcontainers dependencies

#### Success Criteria:
- MongoDB container starts successfully
- Can connect to the container and execute basic commands
- Container stops cleanly after tests

### Step 4.3: Integrate Container with Test Infrastructure
**Goal**: Connect the MongoDB container to our existing test utilities

#### Tasks:
- [ ] Update `TestContext` to use the containerized MongoDB
- [ ] Modify connection string generation for dynamic ports
- [ ] Implement proper container lifecycle in test setup/teardown
- [ ] Add error handling for container startup failures

#### Files to Modify:
- `/price/tests/common/mod.rs` - Update TestContext
- Add container management utilities

#### Success Criteria:
- TestContext successfully uses containerized MongoDB
- Dynamic port handling works correctly
- Test isolation is maintained between test runs

### Step 4.4: Create Integration Test Suite
**Goal**: Implement comprehensive integration tests using the containerized MongoDB

#### Tasks:
- [ ] Create GetBestOfferPrice integration tests with real data
- [ ] Test the complete MongoDB query pipeline from playground-1.mongodb.js
- [ ] Implement test data fixtures and cleanup
- [ ] Add performance baseline tests

#### Files to Create:
- `/price/tests/integration_tests_mongo.rs` - Full integration test suite
- Test fixtures with realistic offer data

#### Success Criteria:
- All integration tests pass against containerized MongoDB
- Test data is properly isolated and cleaned up
- Query performance is within acceptable ranges

### Step 4.5: Handle Edge Cases and Error Scenarios
**Goal**: Ensure robust testing of error conditions and edge cases

#### Tasks:
- [ ] Test MongoDB connection failures
- [ ] Test malformed query scenarios
- [ ] Implement timeout handling for slow queries
- [ ] Add tests for concurrent access scenarios

#### Success Criteria:
- Error scenarios are properly tested
- Timeouts and connection issues are handled gracefully
- Concurrent test execution works correctly

### Step 4.6: Optimize and Document
**Goal**: Finalize the integration test infrastructure with proper documentation

#### Tasks:
- [ ] Optimize container startup time
- [ ] Add comprehensive documentation for running integration tests
- [ ] Create CI/CD friendly test execution
- [ ] Document troubleshooting common issues

#### Files to Create/Modify:
- `/price/tests/README.md` - Integration test documentation
- Update main project documentation

#### Success Criteria:
- Integration tests run efficiently
- Clear documentation for developers
- CI/CD ready test execution

## 🔧 Technical Approach

### Container Configuration Strategy
```rust
// Planned container setup approach
pub struct MongoContainer {
    image: String,
    port: u16,
    env_vars: HashMap<String, String>,
    startup_timeout: Duration,
}

impl MongoContainer {
    pub fn new() -> Self {
        Self {
            image: "mongo:6.0".to_string(),
            port: 27017,
            env_vars: HashMap::new(),
            startup_timeout: Duration::from_secs(30),
        }
    }
}
```

### Test Isolation Strategy
- Each test gets a fresh database instance
- Test data is scoped to unique collection names
- Proper cleanup after each test execution
- Connection pooling for performance

### Error Handling Strategy
- Graceful degradation when containers fail to start
- Clear error messages for debugging
- Retry mechanisms for transient failures
- Timeout handling for slow operations

## 📊 Success Metrics

### Performance Targets
- Container startup time: < 10 seconds
- Test execution time: < 30 seconds for full suite
- Query response time: < 100ms for typical queries

### Quality Targets
- 100% test coverage for GetBestOfferPrice scenarios
- Zero test flakiness due to container issues
- Proper isolation between test runs

## 🚧 Risk Mitigation

### Identified Risks
1. **Container Startup Failures**: Implement retry logic and fallback strategies
2. **Port Conflicts**: Use dynamic port allocation
3. **Resource Cleanup**: Ensure containers are always cleaned up, even on test failures
4. **CI/CD Integration**: Plan for container runtime availability in CI environments

### Mitigation Strategies
- Implement comprehensive error handling
- Add logging for debugging container issues
- Create fallback to local MongoDB for development
- Document system requirements clearly

## 🔄 Implementation Order

1. **Start with Step 4.1**: Research and understand current testcontainers capabilities
2. **Proof of Concept**: Implement Step 4.2 to validate basic approach
3. **Incremental Integration**: Steps 4.3-4.4 to build working test suite
4. **Polish and Optimize**: Steps 4.5-4.6 for production readiness

## 📝 Definition of Done

Phase 4 will be considered complete when:
- [ ] MongoDB testcontainers integration works reliably
- [ ] Full integration test suite passes consistently
- [ ] Documentation is comprehensive and clear
- [ ] CI/CD integration is possible
- [ ] Error scenarios are properly handled
- [ ] Performance targets are met

## 🎯 Next Action
Begin with **Step 4.1** - Research current testcontainers MongoDB support and document the correct API usage patterns for version 0.15.0.

---

*This subplan breaks down the testcontainers integration into manageable, verifiable steps to ensure we achieve true integration testing capabilities for the GetBestOfferPrice API.*
