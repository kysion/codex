#!/usr/bin/env bash
set -euo pipefail

INSTALL_PATH="/usr/local/bin/codex"

choose_backup() {
  if [[ ! -d "/usr/local/bin" ]]; then
    echo "[错误] 未找到 /usr/local/bin 目录。" >&2
    exit 1
  fi

  mapfile -t backups < <(ls -1t /usr/local/bin/codex.bak.* 2>/dev/null || true)
  if (( ${#backups[@]} == 0 )); then
    echo "[错误] 未找到任何 codex.bak.* 备份文件。" >&2
    exit 1
  fi

  echo "可用备份："
  local idx=1
  for file in "${backups[@]}"; do
    echo "  [$idx] $file"
    ((idx++))
  done

  read -r -p "请选择要恢复的备份编号 (默认 1): " choice || choice=""
  if [[ -z "${choice}" ]]; then
    choice=1
  fi

  if ! [[ "${choice}" =~ ^[0-9]+$ ]] || (( choice < 1 || choice > ${#backups[@]} )); then
    echo "[错误] 无效的选择" >&2
    exit 1
  fi

  SELECTED_BACKUP="${backups[choice-1]}"
}

restore_backup() {
  echo "将恢复备份：${SELECTED_BACKUP} -> ${INSTALL_PATH}"
  sudo cp "${SELECTED_BACKUP}" "${INSTALL_PATH}"
  sudo chmod +x "${INSTALL_PATH}"
  echo "恢复完成。"
}

main() {
  choose_backup
  restore_backup
}

main "$@"
