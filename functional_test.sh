# Functional test for transaction-rs
# Checks that the command works with a csv as input and outputs a new csv with the correct data

# Generate an input csv
cat <<EOT >> functional_test.csv
type,client,tx,amount
deposit,1,1,1.0001
deposit,2,2,2.0
deposit,1,3,2.9999
withdrawal,1,  4,  1.5
withdrawal,2,5,3.0
EOT

# Run the command on the input csv
cargo run -- functional_test.csv > accounts.csv

# Check that the output file exists and has the right data
