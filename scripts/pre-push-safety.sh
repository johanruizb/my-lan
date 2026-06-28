#!/usr/bin/env bash
#
# scripts/pre-push-safety.sh — Publish-safety gate para MyLAN.
#
# Bloquea el push si los cambios a publicar contienen:
#   - MACs reales (`([0-9a-fA-F]{2}:){5}[0-9a-fA-F]{2}`) fuera de tests/fixturas
#     y módulos `#[cfg(test)]`.
#   - Secretos obvios (API keys, tokens, private keys).
#
# NO se instala automáticamente. Para usarlo como pre-push hook:
#   - `git config core.hooksPath .githooks` tras copiarlo a `.githooks/pre-push`, o
#   - `cp scripts/pre-push-safety.sh .git/hooks/pre-push && chmod +x .git/hooks/pre-push`
# También se puede ejecutar a mano: `scripts/pre-push-safety.sh` revisa lo staged.
#
# Salida: 0 = limpio, 1 = se hallaron MACs/secretos a publicar.
set -euo pipefail

# Excluir rutas (datos locales, build output, fixtures de test, firmas OUI/reglas).
EXCLUDE_RE='(^|/)(tests/|test/|fixtures/|node_modules/|dist/|target/|gen/|\.omc/|\.serena/)|(^|/)(signatures/)|\.lock$|\.db($|-|-wal$|-shm$|-journal$)|mylan-devices\.(json|csv)$|^\.env$|\.local\.toml$'

# MACs placeholder canónicas (no son de red real): se permiten siempre.
PLACEHOLDER_MAC_RE='^(aa:bb:cc:dd:ee:ff|00:00:00:00:00:00|ff:ff:ff:ff:ff:ff)$'

MAC_RE='([0-9a-fA-F]{2}:){5}[0-9a-fA-F]{2}'

# Secretos obvios: `KEY=...`, `token: ...`, AWS access-key id, bloques private key.
SECRET_RE='(api[_-]?key|secret|access[_-]?token|auth[_-]?token|private[_-]?key|password|passwd|pwd|token)[[:space:]]*[:=][[:space:]]*['"'"'"]?[A-Za-z0-9+/=._-]{16,}|AKIA[0-9A-Z]{16}|-----BEGIN [A-Z ]*PRIVATE KEY-----'

violations=0
violation_log=""

# Emite `<path>\t<new-file-line>\t<added-line-sans-plus>` para cada línea añadida
# del rango diff dado. `--unified=0` => sólo `+`/`-` sin contexto; el contador de
# línea nueva arranca en `s` del header `@@ -a +s,n @@` y avanza por cada `+`.
emit_added_lines() {
	local range="$1"
	git diff --unified=0 --no-color "$range" 2>/dev/null | awk -F'\t' '
		/^diff --git / { sub(/^diff --git a\//,""); sub(/ b\/.*/,""); path=$0; next }
		/^@@ / {
			match($0, /\+([0-9]+)(,[0-9]+)? @@@/);
			s = substr($0, RSTART+1, RLENGTH-5); sub(/,.*/, "", s);
			newln = s; next
		}
		/^\+[^+]/ { sub(/^\+/,""); print path "\t" newln "\t" $0; newln++ }
		/^\+\+\+/ { next }
		/^-/    { next }
	'
}

# ¿La línea `newln` del archivo `path` (versión index/newrev) cae dentro de un
# bloque `#[cfg(test)]`? Heurística: está dentro si `newln` >= línea del primer
# `#[cfg(test)]` del archivo (los mod tests van al final del .rs en este repo).
in_cfg_test_region() {
	local path="$1" newln="$2" blob="$3"
	# blob es la referencia git del contenido nuevo (ej. ":path" para staged,
	# "<newrev>:path" para commits pusheados).
	local cfg_line
	cfg_line=$(git show "$blob" 2>/dev/null | grep -nE '^#\[cfg\(test\)\]' | head -1 | cut -d: -f1 || true)
	[ -n "$cfg_line" ] && [ "$newln" -ge "$cfg_line" ]
}

check_line() {
	local path="$1" line="$2" content="$3" blob="$4"

	# Excluir rutas (datos/build/fixtures/firmas).
	if printf '%s' "$path" | grep -qE "$EXCLUDE_RE"; then return 0; fi

	# MAC real.
	if printf '%s' "$content" | grep -qE "$MAC_RE"; then
		# Extrae cada MAC hallado y decide por uno.
		while IFS= read -r mac; do
			# Placeholder canónico -> permitir.
			if printf '%s' "$mac" | grep -qiE "$PLACEHOLDER_MAC_RE"; then continue; fi
			# Dentro de `#[cfg(test)]` en .rs -> permitir.
			if printf '%s' "$path" | grep -qE '\.rs$'; then
				if in_cfg_test_region "$path" "$line" "$blob"; then continue; fi
			fi
			violation_log+="MAC real ${mac} en ${path}:${line}: ${content}"$'\n'
			violations=$((violations+1))
		done < <(printf '%s' "$content" | grep -oE "$MAC_RE")
	fi

	# Secreto obvio.
	if printf '%s' "$content" | grep -qE "$SECRET_RE"; then
		violation_log+="posible secreto en ${path}:${line}: ${content}"$'\n'
		violations=$((violations+1))
	fi
}

main() {
	local ranges=()
	# Modo pre-push: stdin trae líneas `<oldrev> <newrev> <refname>`.
	if [ ! -t 0 ]; then
		while read -r oldrev newrev refname; do
			# Borrar rama (newrev todo ceros) -> nada que publicar.
			case "$newrev" in *[!0]*) ;; *) continue ;; esac
			case "$oldrev" in
				*[!0]*)
					ranges+=("${oldrev}..${newrev}")
					;;
				*)
					# Rama nueva: commits de newrev que no están en ninguna ref existente.
					ranges+=("$(git rev-list --reverse "$newrev" --not --all --remotes 2>/dev/null | tr '\n' ' ')")
					;;
			esac
		done
	fi

	# Modo manual/staged: revisar lo staged (si no vino stdin como pre-push).
	if [ "${#ranges[@]}" -eq 0 ]; then
		ranges+=("--cached")
	fi

	local range blob_kind
	for range in "${ranges[@]}"; do
		[ -z "$range" ] && continue
		# Para mapear línea -> #[cfg(test)] necesitamos el contenido nuevo.
		# --cached => blob ":<path>"; rango A..B => blob "<newrev-side>:<path>".
		case "$range" in
			--cached) blob_kind="cached" ;;
			*..*)     blob_kind="range:$range" ;;
			*)        blob_kind="commits:$range" ;;
		esac

		# Recorremos las líneas añadidas. Para `#[cfg(test)]` necesitamos el path
		# y el newrev; para rangos usamos el lado nuevo del diff.
		while IFS=$'\t' read -r path line content; do
			[ -z "$path" ] && continue
			case "$blob_kind" in
				cached) blob=":$path" ;;
				range:*) blob="$(printf '%s' "$blob_kind" | sed 's/^range://; s/^[^.]*\.\.//'):$path" ;;
				commits:*) blob="HEAD:$path" ;;  # mejor esfuerzo: estado final del commit
			esac
			check_line "$path" "$line" "$content" "$blob"
		done < <(emit_added_lines "$range")
	done

	if [ "$violations" -ne 0 ]; then
		printf '❌ pre-push-safety: %d hallazgo(s) bloquea(n) el push:\n\n%s\n' \
			"$violations" "$violation_log" >&2
		printf 'Si son falsos positivos (p.ej. MAC sintética documentada), añade el\n' >&2
		printf 'valor a PLACEHOLDER_MAC_RE o marca la línea como test fixture.\n' >&2
		exit 1
	fi

	printf '✓ pre-push-safety: sin MACs reales ni secretos en los cambios a publicar.\n'
	exit 0
}

main "$@"