# Transaction-rs
Simple toy payment engine.

### Assumptions.
Client accounts can not hold negative sums, meaning that if a previous deposit is disputed after one or more withdrawals occurred 
it would be possible that there aren't enough funds available. In that case the dispute is ignored and a warning is emitted.

### Run:
`cargo run -- example.csv > accounts.csv`

### Unit-tests:
`cargo test`

### Functional-tests (inputs and outputs):
`./functional_test.sh `

### Todo:
- add unit tests for missing transaction types
