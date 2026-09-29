# Requirements Document: Deposit Simulation Environment

## Introduction

The Deposit Simulation Environment feature extends the SAFE-HAVEN Stellar blockchain deposit contract to provide a safe testing ground for deposit strategies. Users require the ability to test deposit operations and analyze outcomes without risking real funds. This feature introduces a simulation mode that maintains complete isolation from production data, allowing users to experiment with different deposit configurations, test withdrawal scenarios, and validate strategies before committing real deposits. Simulations are clearly marked and separated from actual operations, with the ability to reset simulation state at any time.

## Glossary

- **Simulation Mode**: A state where the contract operates independently, storing test data separately from production data
- **Simulation Flag**: A boolean indicator in contract state marking whether currently active operations target simulation or production storage
- **Simulation Storage**: Separate persistent storage namespace used exclusively for simulation deposits and their lifecycle
- **Production Storage**: The standard persistent storage used for real, committed deposits and operations
- **Simulation Results**: Aggregated data from a completed or ongoing simulation including deposit count, total amounts, fees calculated, and withdrawal outcomes
- **Depositor**: An account address that initiates deposit operations
- **Deposit ID**: A unique monotonic identifier for each deposit within a depositor's sequence
- **Reset Operation**: An administrative action that clears all simulation storage for a depositor or globally, returning the system to clean state
- **Simulation Metadata**: Information about a simulation including its creation time, deposit count, and current status
- **Isolation Property**: The guarantee that operations in simulation mode do not affect production storage and vice versa

## Requirements

### Requirement 1: Simulation Mode Initialization

**User Story:** As a contract user, I want to enable simulation mode for my account, so that I can safely test deposit strategies without affecting my real deposits.

#### Acceptance Criteria

1. THE User SHALL be able to initialize simulation mode for their account via a contract function call
2. WHEN simulation mode is initialized for a depositor, THE System SHALL create an independent simulation environment with unique storage for that depositor
3. THE System SHALL mark the simulation environment as active in contract state
4. WHERE simulation mode is enabled, THE System SHALL store a simulation flag indicating the active mode
5. WHEN a depositor initializes simulation mode and already has an active simulation, THE System SHALL return an error rather than creating a duplicate environment
6. THE System SHALL NOT modify or access any production deposits when initializing simulation mode

### Requirement 2: Simulated Deposit Operations

**User Story:** As a developer testing strategies, I want to execute deposit operations in simulation mode, so that I can validate my deposit logic without consuming real funds.

#### Acceptance Criteria

1. WHEN a depositor calls simulate_deposit() with a token, amount, and lock duration, THE Simulator SHALL create a test deposit entry
2. THE Simulator SHALL assign a unique deposit ID to each simulated deposit following the same monotonic sequencing as production deposits
3. THE Simulator SHALL store the simulated deposit in simulation storage, completely isolated from production storage
4. WHEN a simulated deposit is created, THE Simulator SHALL record the token type, amount, unlock time, and applicable penalty basis points
5. WHILE a simulation is active, THE Simulator SHALL not write to production deposit storage
6. IF an invalid deposit amount or duration is provided, THEN THE Simulator SHALL return an error without creating the deposit
7. WHERE a depositor has multiple open simulation deposits, THE Simulator SHALL maintain all deposits with distinct IDs in the same simulation namespace

### Requirement 3: Simulated Withdrawal Operations

**User Story:** As a user testing strategies, I want to withdraw from simulated deposits, so that I can verify my full deposit lifecycle works as expected.

#### Acceptance Criteria

1. WHEN a depositor calls simulate_withdraw() with a deposit ID, THE Simulator SHALL retrieve the corresponding simulated deposit
2. IF the specified deposit ID does not exist in simulation storage, THEN THE Simulator SHALL return an error
3. WHEN a withdrawal is executed, THE Simulator SHALL verify the unlock condition (time-based or ledger-based) matches the actual unlock logic
4. THE Simulator SHALL calculate fees and penalties identically to production withdrawals
5. WHEN a simulated withdrawal succeeds, THE Simulator SHALL remove the deposit from simulation storage
6. WHILE simulation storage is isolated, THE Simulator SHALL not access or affect any production deposits during withdrawal

### Requirement 4: Simulation Storage Isolation

**User Story:** As a system administrator, I want simulation and production data stored separately, so that I can guarantee simulation operations never affect real user funds.

#### Acceptance Criteria

1. THE System SHALL use distinct storage namespaces for simulation and production deposits
2. WHEN simulation mode is active for a depositor, THE System SHALL write to simulation storage only
3. WHEN production mode is active, THE System SHALL write to production storage only
4. IF a depositor queries their deposits, THE System SHALL return only deposits from the currently active mode (simulation or production)
5. WHEN simulation mode is deactivated, THE System SHALL not expose simulation data through production queries
6. THE System SHALL maintain separate active deposit ID lists for simulation and production modes
7. IF storage access occurs without the correct isolation context, THEN THE System SHALL return an error

### Requirement 5: Query Simulated Deposits

**User Story:** As a user validating my strategy, I want to query simulated deposits, so that I can verify the state and details of my test operations.

#### Acceptance Criteria

1. WHEN a depositor calls get_simulation_results() during an active simulation, THE System SHALL return current simulation state and metrics
2. THE System SHALL return all active simulated deposit IDs in a single query
3. THE System SHALL include in results: total number of active deposits, total amount locked, earliest unlock time, and penalty fees applied
4. WHEN querying individual simulated deposits, THE System SHALL return deposit details (token, amount, unlock time, penalty) from simulation storage
5. IF simulation mode is not active, THEN THE System SHALL return an error when querying simulation results
6. THE System SHALL support paginated queries of simulated deposits analogous to production deposit queries
7. WHERE multiple simulated deposits exist, THE System SHALL return all results consistently without loss or corruption

### Requirement 6: Reset Simulation State

**User Story:** As a user iterating on strategies, I want to reset my simulation, so that I can start fresh with clean test data.

#### Acceptance Criteria

1. WHEN a depositor calls reset_simulation(), THE System SHALL clear all simulation deposits for that depositor
2. THE System SHALL remove all active simulation deposit IDs for the depositor
3. WHEN simulation is reset, THE System SHALL decrement simulation deposit counters to zero
4. THE System SHALL leave simulation mode active and ready for new deposits after a reset
5. WHEN a reset occurs, THE System SHALL preserve all production deposits without any modification
6. IF a depositor has no active simulation, THE System SHALL return an error rather than proceeding with reset
7. THE System SHALL provide an optional admin function to reset simulation state globally across all depositors

### Requirement 7: Simulation Metadata and Status Tracking

**User Story:** As a system operator, I want to track simulation metadata, so that I understand which simulations are active and gather diagnostic information.

#### Acceptance Criteria

1. WHEN a simulation is initialized, THE System SHALL record the creation timestamp
2. THE System SHALL store the number of deposits created in the current simulation
3. WHEN querying simulation status, THE System SHALL return active simulation count and recent simulation activity
4. THE System SHALL record the last modification time for each simulation
5. WHERE a depositor has an active simulation, THE System SHALL mark that simulation as active in metadata
6. IF simulation metadata is corrupted or inconsistent, THEN THE System SHALL return an error during queries

### Requirement 8: Clear Visual and Operational Distinction

**User Story:** As a user, I want clear indication of whether operations are in simulation or production mode, so that I never accidentally confuse test data with real funds.

#### Acceptance Criteria

1. THE System SHALL include a "simulation_mode" field in all contract state queries indicating the current mode
2. WHEN a depositor initiates a deposit operation, THE System SHALL return the mode (simulation or production) in the transaction response
3. WHEN simulation mode is active, THE System SHALL prepend "SIM_" to all simulation-specific event types
4. WHERE a depositor queries their deposits, THE System SHALL label each result with its origin (simulation or production)
5. THE System SHALL prohibit mixing simulation and production operations in a single transaction batch
6. IF a user attempts to withdraw from a production deposit using a simulation withdrawal function, THEN THE System SHALL return an error

### Requirement 9: Simulation Performance and Constraints

**User Story:** As a contract maintainer, I want simulation operations to perform efficiently, so that users experience reasonable query and transaction latencies.

#### Acceptance Criteria

1. WHEN a simulation deposit is created, THE System SHALL complete the operation in constant time O(1) storage writes
2. WHEN retrieving simulation results, THE System SHALL complete the query in O(1) time when accessing metadata
3. WHEN retrieving all active simulation deposit IDs, THE System SHALL complete in O(n-active) time where n-active is the count of active simulated deposits
4. WHEN resetting a simulation, THE System SHALL complete the operation in O(n-active) time proportional to the number of active deposits
5. THE System SHALL impose no artificial limitation on the number of concurrent simulations across different depositors
6. THE System SHALL store simulation metadata efficiently without creating per-deposit overhead beyond production deposits
7. IF a simulation operation exceeds transaction budget constraints, THEN THE System SHALL return an error with clear indication of the constraint violation

### Requirement 10: Simulation and Production Coexistence

**User Story:** As a user, I want to maintain both simulation and production deposits simultaneously, so that I can test new strategies while keeping my existing real deposits active.

#### Acceptance Criteria

1. WHILE a depositor has simulation mode active, THE System SHALL not restrict the depositor's ability to perform production deposit operations
2. WHEN a depositor performs a production deposit, THE System SHALL write to production storage without affecting simulation storage
3. WHERE both simulations and production deposits exist, THE System SHALL maintain separate ID sequences for each mode
4. WHEN querying deposits, THE System SHALL allow the depositor to explicitly specify which mode to query
5. IF a depositor has active deposits in both modes, THE System SHALL track and report each mode independently
6. THE System SHALL not allow operations in one mode to increment counters or IDs in the other mode

### Requirement 11: Error Handling and Validation

**User Story:** As a developer integrating the simulation feature, I want clear error messages and validation, so that I can quickly diagnose issues and fix my code.

#### Acceptance Criteria

1. WHEN an invalid operation is attempted in simulation mode, THE System SHALL return a descriptive error code
2. IF a depositor attempts to access a non-existent simulation, THEN THE System SHALL return a "SimulationNotFound" error
3. IF storage isolation is violated, THEN THE System SHALL return an "IsolationViolation" error
4. WHEN a depositor attempts to reset a non-existent simulation, THE System SHALL return a "SimulationNotFound" error
5. WHERE constraints are exceeded (e.g., too many deposits), THE System SHALL provide specific error messages indicating which constraint
6. THE System SHALL validate all inputs (token address, amounts, durations) before creating simulation deposits

### Requirement 12: Simulation Mode Deactivation

**User Story:** As a user ready to commit real deposits, I want to disable simulation mode, so that subsequent operations default to production mode.

#### Acceptance Criteria

1. WHEN a depositor calls disable_simulation_mode(), THE System SHALL clear the simulation flag for that depositor
2. WHEN simulation mode is disabled, THE System SHALL preserve all simulation data for historical reference if queried
3. THE System SHALL require an explicit enable call to reactivate simulation mode
4. AFTER disabling simulation mode, THE System SHALL route all new operations to production storage
5. IF simulation mode is not currently active, THE System SHALL return an error when attempting to disable it
6. WHERE simulation data exists, THAT data SHALL remain queryable until explicitly reset

## Non-Functional Requirements

### NFR 1: Storage Efficiency
The simulation feature SHALL not significantly increase contract size or per-depositor storage overhead beyond the production deposit storage model.

### NFR 2: Backward Compatibility
The simulation feature SHALL not modify or break existing production deposit operations or their storage layout.

### NFR 3: Access Control
All simulation operations SHALL respect the existing authorization model (requiring the depositor's signature or admin authorization).

### NFR 4: Determinism
Simulation operations SHALL produce deterministic results independent of call order when applied to the same starting state.

