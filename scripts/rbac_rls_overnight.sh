#!/bin/sh
set -u
umask 077

run_dir=${E2BR3_OVERNIGHT_ARTIFACT_DIR:-tmp/rbac-rls-fuzz/overnight}
duration=${E2BR3_OVERNIGHT_SECONDS:-17000}
interval=${E2BR3_OVERNIGHT_INTERVAL:-20}
max_actions=${E2BR3_MAX_ACTIONS:-100}
base_url=${E2BR3_BASE_URL:-http://127.0.0.1:8080}
mkdir -p "$run_dir"

orchestrator_log="$run_dir/orchestrator.jsonl"
runner_log="$run_dir/runner.log"
started=$(date +%s)
deadline=$((started + duration))
seed=$started
failures=0
stop_status=DEADLINE

while [ "$(date +%s)" -lt "$deadline" ]; do
	health=$(curl -sS -o /dev/null -w '%{http_code}' --max-time 5 "$base_url/health" 2>/dev/null || printf '000')
	if [ "$health" != "204" ]; then
		stop_status=HEALTH_BLOCKED
		printf '{"kind":"stop","status":"INCONCLUSIVE","reason":"health","http":%s,"seed":%s}\n' "$health" "$seed" >> "$orchestrator_log"
		break
	fi

	E2BR3_BASE_URL="$base_url" python3 scripts/rbac_rls_blackbox.py \
		--seed "$seed" --max-actions "$max_actions" \
		--artifact-dir "$run_dir" matrix >> "$runner_log" 2>&1
	rc=$?
	printf '{"kind":"iteration","seed":%s,"exit":%s}\n' "$seed" "$rc" >> "$orchestrator_log"
	if [ "$rc" -eq 1 ]; then
		failures=$((failures + 1))
		if [ "$failures" -ge 3 ]; then
			stop_status=ERROR_THRESHOLD
			printf '{"kind":"stop","status":"INCONCLUSIVE","reason":"error_threshold","seed":%s}\n' "$seed" >> "$orchestrator_log"
			break
		fi
	elif [ "$rc" -eq 2 ]; then
		stop_status=INCONCLUSIVE
		printf '{"kind":"stop","status":"INCONCLUSIVE","reason":"runner_stop","seed":%s}\n' "$seed" >> "$orchestrator_log"
		break
	else
		failures=0
	fi
	seed=$((seed + 1))
	sleep "$interval"
done

printf '{"kind":"complete","status":"%s","started":%s,"deadline":%s,"last_seed":%s}\n' \
	"$stop_status" "$started" "$deadline" "$seed" >> "$orchestrator_log"
