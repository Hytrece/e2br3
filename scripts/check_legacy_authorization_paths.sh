#!/usr/bin/env bash
set -euo pipefail

mode="${1:---report}"
if [[ "$mode" != "--report" && "$mode" != "--enforce-zero" ]]; then
	printf 'usage: %s [--report|--enforce-zero]\n' "$0" >&2
	exit 2
fi

# RLS context setters are storage isolation primitives used only after an
# authorization decision. This gate targets competing authorization APIs.
pattern='has_permission|require_permission|ctx\.is_admin|RequireAdmin|require_admin|can_access_user_admin|permission_contract|can_modify'
matches="$(rg -n "$pattern" crates/libs crates/services/web-server/src \
	crates/services/web-server/examples \
	-g '!**/tests/**' \
	-g '!**/target/**' || true)"

if [[ -n "$matches" ]]; then
	printf '%s\n' "$matches"
	if [[ "$mode" == "--enforce-zero" ]]; then
		exit 1
	fi
fi
