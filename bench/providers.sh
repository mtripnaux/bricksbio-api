#!/bin/bash
# bench/providers.sh: Bench all providers for all ids in all CSVs in bench/providers/

set -euo pipefail

if ! curl -s --connect-timeout 2 http://localhost:3000/ > /dev/null; then
    echo "[ERREUR] L'API n'est pas accessible sur http://localhost:3000. Lancez le serveur avant de lancer ce script." >&2
    exit 1
fi

printf "%-20s %-20s %-10s %-10s\n" "CSV" "ID" "Status" "Time(ms)"

csv_count=0
for csv in $(find ./bench/providers -name '*.csv'); do
    csv_count=$((csv_count+1))
    line_count=0
    while IFS= read -r id; do
        [[ -z "$id" ]] && continue
        line_count=$((line_count+1))
        start=$(date +%s%3N)
        response=$(curl -s -o /dev/null -w "%{http_code}" "http://localhost:3000/parts/$id")
        status=$?
        code=$response
        end=$(date +%s%3N)
        elapsed=$((end - start))
        if [[ $status -eq 0 ]]; then
            printf "%-20s %-20s %-10s %-10s\n" "$(basename $csv)" "$id" "$code" "$elapsed"
        else
            printf "%-20s %-20s %-10s %-10s\n" "$(basename $csv)" "$id" "ERR" "$elapsed"
        fi
    done < "$csv"
done