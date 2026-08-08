#!/bin/sh
set -u
umask 077

run_dir=${E2BR3_OVERNIGHT_ARTIFACT_DIR:-tmp/rbac-rls-fuzz/overnight-adversarial}
duration=${E2BR3_OVERNIGHT_SECONDS:-17000}
interval=${E2BR3_OVERNIGHT_INTERVAL:-20}
max_actions=${E2BR3_MAX_ACTIONS:-800}
matrix_rounds=${E2BR3_MATRIX_ROUNDS:-16}
adversarial_actions=${E2BR3_ADVERSARIAL_ACTIONS:-64}
base_url=${E2BR3_BASE_URL:-http://127.0.0.1:8080}
mkdir -p "$run_dir"

orchestrator_log="$run_dir/orchestrator.jsonl"
runner_log="$run_dir/runner.log"
started=$(date +%s)
deadline=$((started + duration))
seed=$started
stop_status=DEADLINE

printf '{"kind":"start","started":%s,"deadline":%s,"matrix_rounds":%s,"adversarial_actions":%s,"max_actions":%s}\n' \
	"$started" "$deadline" "$matrix_rounds" "$adversarial_actions" "$max_actions" > "$orchestrator_log"

while [ "$(date +%s)" -lt "$deadline" ]; do
	health=$(curl -sS -o /dev/null -w '%{http_code}' --max-time 5 "$base_url/health" 2>/dev/null || printf '000')
	if [ "$health" != "204" ]; then
		stop_status=HEALTH_BLOCKED
		printf '{"kind":"stop","status":"INCONCLUSIVE","reason":"health","http":%s,"seed":%s}\n' "$health" "$seed" >> "$orchestrator_log"
		break
	fi

	E2BR3_BASE_URL="$base_url" python3 scripts/rbac_scope_stateful.py \
		--seed "$seed" --max-actions "$max_actions" --matrix-rounds "$matrix_rounds" \
		--adversarial-actions "$adversarial_actions" --deadline-seconds 240 \
		--artifact-dir "$run_dir" >> "$runner_log" 2>&1
	rc=$?
	printf '{"kind":"iteration","seed":%s,"exit":%s}\n' "$seed" "$rc" >> "$orchestrator_log"
	if [ "$rc" -eq 1 ]; then
		stop_status=ERROR_THRESHOLD
		printf '{"kind":"stop","status":"INCONCLUSIVE","reason":"fail","seed":%s}\n' "$seed" >> "$orchestrator_log"
		break
	fi
	if [ "$rc" -eq 2 ]; then
		stop_status=INCONCLUSIVE
		printf '{"kind":"stop","status":"INCONCLUSIVE","reason":"runner_stop","seed":%s}\n' "$seed" >> "$orchestrator_log"
		break
	fi
	if [ "$rc" -ne 0 ]; then
		stop_status=ERROR
		printf '{"kind":"stop","status":"INCONCLUSIVE","reason":"runner_exit","exit":%s,"seed":%s}\n' "$rc" "$seed" >> "$orchestrator_log"
		break
	fi

	seed=$((seed + 1))
	sleep "$interval"
done

printf '{"kind":"complete","status":"%s","started":%s,"deadline":%s,"last_seed":%s}\n' \
	"$stop_status" "$started" "$deadline" "$seed" >> "$orchestrator_log"
