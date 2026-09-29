#!/bin/bash
cd "$(dirname "$0")"
jobs_list=()
for c in 1748 2908 3191; do for arm in base hook; do for t in 1 2; do jobs_list+=("$c $arm $t"); done; done; done
printf '%s\n' "${jobs_list[@]}" | xargs -P 6 -L 1 bash -c 'python3 run_case.py $0 $1 $2 > logs/$0-$1-t$2.out 2> logs/$0-$1-t$2.err; echo "done $0 $1 $2 rc=$?"'
