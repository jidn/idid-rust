#!/bin/bash
 
# Example 1: Show total of todays activity.
#   $ idid show 0 | grep -v "lunch" | ./group-by-day.sh
#   2024-04-01	07:41
# Example 2: Show last weeks daily total.
#   $ idid show mon fri | grep -v "lunch" | sort | ./group-by-day.sh
#   2024-03-25	07:55
#   2024-03-26	08:11
#   2024-03-27	07:58
#   2024-03-28	08:02
#   2024-03-29	07:56

# Initialize variables
current_day=""
total_duration="00:00"

# Read input from stdin
while IFS=$'\t' read -r datetime duration text; do
    # Extract the day from the RFC 3339 date time
    day=$(date -d "$datetime" +%Y-%m-%d)

    # If it's a new day, print the total duration for the previous day and reset variables
    if [[ "$day" != "$current_day" && "$current_day" != "" ]]; then
        echo -e "$current_day\t$total_duration"
        total_duration="00:00"
        total_duration_seconds=0
    fi

    # Add the duration to the total for the current day
    duration_in_minutes=$(date -d "1970-01-01 $duration UTC" +%s) # Convert duration to seconds since epoch
    total_duration_seconds=$((total_duration_seconds + duration_in_minutes))
    total_duration=$(date -u -d "@$total_duration_seconds" +%H:%M) # Format total duration as HH:MM

    # Update current day
    current_day="$day"
done

# Print the total duration for the last day
echo -e "$current_day\t$total_duration"

