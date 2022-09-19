# Transaction-rs
Simple toy payment engine.

# Assumptions.
Client accounts can hold negative sums, if a previous deposit is disputed after withdrawals occurred.
If client accounts can not hold negative sums, the dispute should be ignored when there aren't enough funds available.


### Run:
`cargo run -- example.csv > accounts.csv`

### Todo:
- add unit tests for missing transaction types
