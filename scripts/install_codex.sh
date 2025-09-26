#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
LOCAL_BINARY="${REPO_ROOT}/codex-rs/target/release/codex"
INSTALL_PATH="/usr/local/bin/codex"
TEMP_BINARY=""
DOWNLOAD_URL_DEFAULT="https://github.com/kysion/codex/releases/latest/download/codex-macos-universal"

cleanup() {
  if [[ -n "${TEMP_BINARY}" && -f "${TEMP_BINARY}" ]]; then
    rm -f "${TEMP_BINARY}" || true
  fi
}
trap cleanup EXIT

require_macos() {
  if [[ "$(uname -s)" != "Darwin" ]]; then
    echo "[错误] 该安装脚本仅支持 macOS。" >&2
    exit 1
  fi
}

try_install_with_brew() {
  local pkg="$1"
  if command -v brew >/dev/null 2>&1; then
    echo "[信息] 使用 Homebrew 安装 ${pkg}…"
    brew install "$pkg"
  else
    echo "[错误] 缺少 ${pkg}，且未检测到 Homebrew，无法自动安装。" >&2
    echo "        请手动安装依赖后重试。" >&2
    exit 1
  fi
}

ensure_dependencies() {
  local need_node=false
  for cmd in node npm npx; do
    if ! command -v "$cmd" >/dev/null 2>&1; then
      need_node=true
      break
    fi
  done
  if [[ "$need_node" == true ]]; then
    try_install_with_brew node
  fi

  if ! command -v python3 >/dev/null 2>&1; then
    try_install_with_brew python
  fi

  for cmd in node npm npx python3; do
    if ! command -v "$cmd" >/dev/null 2>&1; then
      echo "[错误] 依赖 ${cmd} 安装失败，请手动安装。" >&2
      exit 1
    fi
  done
}

ensure_curl_or_wget() {
  if command -v curl >/dev/null 2>&1; then
    DOWNLOADER="curl"
  elif command -v wget >/dev/null 2>&1; then
    DOWNLOADER="wget"
  else
    try_install_with_brew curl
    DOWNLOADER="curl"
  fi
}

build_or_download_binary() {
  if [[ -f "${LOCAL_BINARY}" ]]; then
    BINARY_SOURCE="${LOCAL_BINARY}"
    return
  fi

  if [[ -d "${REPO_ROOT}/codex-rs" ]] && command -v cargo >/dev/null 2>&1; then
    echo "[信息] 未找到发布版二进制，尝试本地构建…"
    (cd "${REPO_ROOT}/codex-rs" && RUSTUP_TOOLCHAIN=stable cargo build --release --bin codex)
    if [[ -f "${LOCAL_BINARY}" ]]; then
      BINARY_SOURCE="${LOCAL_BINARY}"
      return
    fi
    echo "[警告] 本地构建失败或仍未产生二进制，将尝试下载。"
  fi

  ensure_curl_or_wget
  local download_url="${CODEX_BINARY_URL:-$DOWNLOAD_URL_DEFAULT}"
  if [[ -z "${download_url}" ]]; then
    echo "[错误] 未提供有效的 CODEX_BINARY_URL，且本地没有可用二进制。" >&2
    exit 1
  fi

  TEMP_BINARY="$(mktemp)"
  echo "[信息] 正在从 ${download_url} 下载 codex…"
  if [[ "${DOWNLOADER}" == "curl" ]]; then
    curl -Lf "${download_url}" -o "${TEMP_BINARY}"
  else
    wget -qO "${TEMP_BINARY}" "${download_url}"
  fi

  if [[ -n "${CODEX_BINARY_SHA256:-}" ]]; then
    echo "[信息] 正在校验 SHA256…"
    local actual
    actual=$(shasum -a 256 "${TEMP_BINARY}" | awk '{print $1}')
    if [[ "${actual}" != "${CODEX_BINARY_SHA256}" ]]; then
      echo "[错误] 校验失败：期待 ${CODEX_BINARY_SHA256}，实际 ${actual}" >&2
      exit 1
    fi
  fi

  chmod +x "${TEMP_BINARY}"
  BINARY_SOURCE="${TEMP_BINARY}"
}

choose_browser() {
  echo "选择用于 MCP 的浏览器："
  read -r -p "[chrome/edge/custom] (默认 chrome): " browser_choice || browser_choice=""
  browser_choice=$(printf '%s' "${browser_choice}" | tr '[:upper:]' '[:lower:]')
  if [[ -z "${browser_choice}" ]]; then
    browser_choice="chrome"
  fi

  case "${browser_choice}" in
    edge)
      BROWSER_PATH="/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge"
      ;;
    chrome)
      BROWSER_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
      ;;
    custom)
      read -r -p "请输入浏览器可执行文件的完整路径: " BROWSER_PATH || BROWSER_PATH=""
      ;;
    *)
      echo "[错误] 未知选项: ${browser_choice}" >&2
      exit 1
      ;;
  esac

  if [[ -z "${BROWSER_PATH}" ]]; then
    echo "[错误] 浏览器路径不能为空" >&2
    exit 1
  fi

  if [[ ! -x "${BROWSER_PATH}" ]]; then
    echo "[警告] 未检测到可执行文件：${BROWSER_PATH}" >&2
    read -r -p "仍要继续并写入该路径? [y/N]: " cont || cont=""
    if [[ "${cont}" != "y" && "${cont}" != "Y" ]]; then
      echo "已取消" >&2
      exit 1
    fi
  fi
}

install_binary() {
  if [[ -f "${INSTALL_PATH}" ]]; then
    local ts="$(date +%Y%m%d%H%M%S)"
    echo "检测到已有 codex，正在备份 -> ${INSTALL_PATH}.bak.${ts}"
    sudo cp "${INSTALL_PATH}" "${INSTALL_PATH}.bak.${ts}"
  fi

  echo "正在复制新的 codex 到 ${INSTALL_PATH} (需要 sudo)…"
  sudo cp "${BINARY_SOURCE}" "${INSTALL_PATH}"
  sudo chmod +x "${INSTALL_PATH}"
  echo "二进制安装完成。"
}

update_config() {
  CONFIG_HOME="${CODEX_HOME:-$HOME/.codex}"
  CONFIG_FILE="${CONFIG_HOME}/config.toml"
  mkdir -p "${CONFIG_HOME}"

  if [[ -f "${CONFIG_FILE}" ]]; then
    cp "${CONFIG_FILE}" "${CONFIG_FILE}.bak.$(date +%Y%m%d%H%M%S)"
  fi

  python3 - "$CONFIG_FILE" "$BROWSER_PATH" <<'PY'
import re
import sys
from pathlib import Path

config_path = Path(sys.argv[1])
browser_path = sys.argv[2]

snippet = f"""
[mcp_servers.chrome]
preset = \"chrome_devtools\"
command = \"npx\"
args = [\"chrome-devtools-mcp@latest\", \"--stdio\", \"--isolated\"]
tool_timeout_sec = 120

[mcp_servers.chrome.env]
CHROME_PATH = \"{browser_path}\"
""".strip() + "\n"

text = config_path.read_text() if config_path.exists() else ""
pattern = re.compile(r"(?sm)^[ \t]*\[mcp_servers\.chrome(?:\.env)?\].*?(?=^[ \t]*\[|\Z)")
text = pattern.sub('', text)
text = text.rstrip() + "\n\n" + snippet + "\n"
config_path.write_text(text)
PY

  CONFIG_FILE_PATH="$CONFIG_FILE"
}

configure_env_exports() {
  local base_url="" api_key=""
  echo
  read -r -p "是否配置 OPENAI_BASE_URL (留空跳过): " base_url || base_url=""
  read -r -s -p "是否配置 OPENAI_API_KEY (留空跳过): " api_key || api_key=""
  echo

  if [[ -z "${base_url}" && -z "${api_key}" ]]; then
    ENV_FILE=""
    return
  fi

  ENV_FILE="${CONFIG_HOME}/env.sh"
  if [[ -f "${ENV_FILE}" ]]; then
    cp "${ENV_FILE}" "${ENV_FILE}.bak.$(date +%Y%m%d%H%M%S)"
  fi

  tmp_file="$(mktemp)"
  if [[ -f "${ENV_FILE}" ]]; then
    grep -v '^export OPENAI_' "${ENV_FILE}" >"${tmp_file}" || true
  fi

  {
    if [[ -n "${base_url}" ]]; then
      echo "export OPENAI_BASE_URL='${base_url}'"
      echo "export OPENAI_API_BASE='${base_url}'"
    fi
    if [[ -n "${api_key}" ]]; then
      echo "export OPENAI_API_KEY='${api_key}'"
    fi
  } >>"${tmp_file}"

  mv "${tmp_file}" "${ENV_FILE}"
}

main() {
  require_macos
  build_or_download_binary
  ensure_dependencies
  choose_browser
  install_binary
  update_config
  configure_env_exports

  echo
  echo "安装完成！"
  echo "- 二进制安装路径：${INSTALL_PATH}"
  echo "- 配置文件：${CONFIG_FILE_PATH}"
  if [[ -n "${ENV_FILE:-}" ]]; then
    echo "- 环境变量文件：${ENV_FILE} (请执行 'source ${ENV_FILE}')"
  else
    echo "- 本次未配置 OPENAI_* 环境变量，如需代理请手动 export。"
  fi
  cat <<'TIP'

验证命令示例：
  codex --sandbox danger-full-access exec "请使用 chrome_devtools 工具打开 https://www.baidu.com 并截取整页截图。"

首次使用 MCP 时，npx 可能下载依赖，请保证网络可达。
TIP
}

main "$@"
