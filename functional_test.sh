# Functional test for transaction-rs
# Checks that the command works with a csv as input and outputs a new csv with the correct data

# Generate an input csv
cat <<EOT > functional_test.csv
type,client,tx,amount
deposit,1,1,1.0001
deposit,2,2,2.0
deposit,1,3,2.9999
withdrawal,1,  4,  1.5
withdrawal,2,5,1.0
dispute, 1, 1,
resolve, 1, 1,
dispute, 1, 1,
dispute, 2, 2,
chargeback, 2, 2,
EOT

# Run the command on the input csv
cargo run -- functional_test.csv > accounts.csv

# Check that the output file exists and has the right data
FILE=accounts.csv
if [ ! -f "$FILE" ]; then
    echo "$FILE does not exist."
    exit 1
fi

