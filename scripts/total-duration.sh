#!/bin/bash

# Example 1: Show total of todays activity.
#   $ idid show 0 | grep -v "lunch" | ./total-duration.sh -t
#   07:41
   
# Example 2: Show last weeks total.
#   $ idid show mon fri | grep -v "lunch" | ./group-by-day.sh | ./total-duration.sh
#   2024-03-29	07:56
#   2024-03-28	08:02
#   2024-03-27	07:58
#   2024-03-26	08:11
#   2024-03-25	07:55
#   Total Duration: 40:02

# Example 2: Show last weeks total duration , but only the total.
#   $ idid show mon fri | grep -v "lunch" | ./total-duration.sh -t
#   40:02

# Initialize variables
total_duration="00:00"
print_total_only=false

# Function to parse and calculate duration in minutes
calculate_duration() {
    duration=$1
    # Convert duration to seconds since epoch
    duration_in_minutes=$(date -d "1970-01-01 $duration UTC" +%s)
    echo "$((duration_in_minutes / 60))"
}

# Function to print usage information
print_usage() {
    echo "Usage: - $0 [-t]"
    echo -e "\nOptions:"
    echo "  -t  Print only the total duration as HH:MM"
    echo -e "\nDescription:"
    echo "  This script takes idid TSV from stdin and computes total duration."
    echo -e "\nExamples:"
    echo "  idid show 0 | grep -v 'lunch' | $0 -t"
    echo "  idid show mon fri | grep -v 'lunch' | ./group-by-day.sh | $0"
}

# Parse command line options
while getopts ":t" opt; do
    case ${opt} in
        t)
            print_total_only=true
            ;;
        \?)
            echo "Error: Invalid option -$OPTARG" >&2
            print_usage
            exit 1
            ;;
    esac
done
shift $((OPTIND -1))

# Read input from stdin
while IFS=$'\t' read -r day duration text; do
    if ! "$print_total_only"; then
        echo -e "$datetime\t$duration\t$text"
    fi
    total_duration_minutes=$((total_duration_minutes + $(calculate_duration "$duration")))
done

total_duration_hours=$((total_duration_minutes / 60))
total_duration_minutes=$((total_duration_minutes % 60))

# Print the total duration in HH:MM format
if "$print_total_only"; then
    printf "%02d:%02d\n" "$total_duration_hours" "$total_duration_minutes"
else
    printf "Total Duration: %02d:%02d\n" "$total_duration_hours" "$total_duration_minutes"
fi
