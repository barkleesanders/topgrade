#!/usr/bin/env bash
# gh-tool-updater.sh — Discover, register, and update GitHub-released tools
#
# Replaces per-tool topgrade custom commands with a single registry-driven updater.
# Uses `gh api` exclusively for all GitHub API calls (authenticated, 5000 req/hr).
#
# Usage:
#   gh-tool-updater.sh update        # Update all registered tools (topgrade calls this)
#   gh-tool-updater.sh scan          # Discover unregistered GitHub tools in PATH
#   gh-tool-updater.sh status        # Show installed vs latest versions
#   gh-tool-updater.sh add           # Interactively add a tool to the registry
#   gh-tool-updater.sh check <name>  # Check a single tool
#
# Registry: ~/.config/gh-tool-registry.json
# Log: ~/.config/gh-tool-updater.log

set -euo pipefail

REGISTRY="${HOME}/.config/gh-tool-registry.json"
LOG="${HOME}/.config/gh-tool-updater.log"
SCAN_DIRS=("${HOME}/.factory/bin" "${HOME}/.local/bin" "/usr/local/bin" "${HOME}/.cargo/bin")

# ─── Colors ─────────────────────────────────────────────────────────

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

# ─── Logging ────────────────────────────────────────────────────────

log() {
    local ts
    ts=$(date '+%Y-%m-%d %H:%M:%S')
    echo "[$ts] $*" >> "$LOG"
}

info()  { echo -e "${BLUE}  [info]${NC} $*"; log "INFO: $*"; }
ok()    { echo -e "${GREEN}  [ok]${NC} $*"; log "OK: $*"; }
warn()  { echo -e "${YELLOW}  [warn]${NC} $*"; log "WARN: $*"; }
fail()  { echo -e "${RED}  [fail]${NC} $*"; log "FAIL: $*"; }
header(){ echo -e "\n${BOLD}${CYAN}$*${NC}"; log "=== $* ==="; }

# ─── Preflight ──────────────────────────────────────────────────────

preflight() {
    if ! command -v gh >/dev/null 2>&1; then
        fail "gh CLI not found. Install: brew install gh"
        exit 1
    fi
    if ! command -v jq >/dev/null 2>&1; then
        fail "jq not found. Install: brew install jq"
        exit 1
    fi
    if ! gh auth status >/dev/null 2>&1; then
        fail "gh not authenticated. Run: gh auth login"
        exit 1
    fi
    if [[ ! -f "$REGISTRY" ]]; then
        fail "Registry not found: $REGISTRY"
        exit 1
    fi
}

# ─── Registry Helpers ───────────────────────────────────────────────

tool_count() {
    jq '.tools | length' "$REGISTRY"
}

get_tool_field() {
    local idx="$1" field="$2"
    jq -r ".tools[$idx].$field // empty" "$REGISTRY"
}

# ─── Version Helpers ────────────────────────────────────────────────

# Normalize version string: strip leading 'v', trim whitespace
normalize_version() {
    local v="$1"
    v="${v#v}"
    v="$(echo "$v" | xargs)"  # trim whitespace
    echo "$v"
}

get_installed_version() {
    local version_command="$1"
    local version_extract="$2"
    local raw

    raw=$(eval "$version_command" 2>/dev/null) || true
    if [[ -z "$raw" ]]; then
        echo ""
        return
    fi

    if [[ -n "$version_extract" && "$version_extract" != "cat" ]]; then
        raw=$(echo "$raw" | eval "$version_extract" 2>/dev/null) || true
    fi

    normalize_version "$raw"
}

get_latest_version() {
    local repo="$1"
    local tag_prefix="$2"
    local tag_filter="$3"
    local tag raw_version

    if [[ -n "$tag_filter" ]]; then
        # Monorepo: list releases and filter by tag pattern
        tag=$(gh release list --repo "$repo" --limit 30 2>/dev/null \
            | grep -E "$tag_filter" \
            | head -1 \
            | awk '{print $1}')
        if [[ -z "$tag" ]]; then
            echo ""
            return
        fi
        # Strip tag_prefix to get version
        raw_version="${tag#$tag_prefix}"
    else
        # Standard repo: latest release
        tag=$(gh release view --repo "$repo" --json tagName -q .tagName 2>/dev/null) || true
        if [[ -z "$tag" ]]; then
            echo ""
            return
        fi
        if [[ -n "$tag_prefix" ]]; then
            raw_version="${tag#$tag_prefix}"
        else
            raw_version="$tag"
        fi
    fi

    normalize_version "$raw_version"
}

get_latest_tag() {
    local repo="$1"
    local tag_filter="$2"

    if [[ -n "$tag_filter" ]]; then
        gh release list --repo "$repo" --limit 30 2>/dev/null \
            | grep -E "$tag_filter" \
            | head -1 \
            | awk '{print $1}'
    else
        gh release view --repo "$repo" --json tagName -q .tagName 2>/dev/null || true
    fi
}

# ─── Install Functions ──────────────────────────────────────────────

install_binary() {
    local repo="$1" tag="$2" asset_pattern="$3" install_path="$4" post_install="$5"
    local temp_dir asset_file

    temp_dir=$(mktemp -d)
    trap "rm -rf '$temp_dir'" RETURN

    info "Downloading $asset_pattern from $repo@$tag..."
    if ! gh release download "$tag" --repo "$repo" --pattern "$asset_pattern" -D "$temp_dir" 2>/dev/null; then
        fail "Download failed for $asset_pattern from $repo@$tag"
        return 1
    fi

    asset_file=$(ls "$temp_dir"/ | head -1)
    if [[ -z "$asset_file" ]]; then
        fail "No asset downloaded"
        return 1
    fi

    chmod +x "$temp_dir/$asset_file"

    # Determine if we need sudo
    local install_dir
    install_dir=$(dirname "$install_path")
    if [[ -w "$install_dir" ]]; then
        mv "$temp_dir/$asset_file" "$install_path"
    else
        sudo mv "$temp_dir/$asset_file" "$install_path"
    fi

    # Run post-install hooks
    if [[ -n "$post_install" ]]; then
        eval "$post_install" || warn "Post-install hook had errors"
    fi

    # Clear quarantine on macOS
    xattr -d com.apple.quarantine "$install_path" 2>/dev/null || true

    return 0
}

install_tarball() {
    local repo="$1" tag="$2" asset_pattern="$3" install_path="$4"
    local archive_binary_path="$5" post_install="$6"
    local temp_dir asset_file binary_file

    temp_dir=$(mktemp -d)
    trap "rm -rf '$temp_dir'" RETURN

    info "Downloading $asset_pattern from $repo@$tag..."
    if ! gh release download "$tag" --repo "$repo" --pattern "$asset_pattern" -D "$temp_dir" 2>/dev/null; then
        fail "Download failed for $asset_pattern from $repo@$tag"
        return 1
    fi

    asset_file=$(ls "$temp_dir"/*.tar.gz 2>/dev/null || ls "$temp_dir"/*.tgz 2>/dev/null || ls "$temp_dir"/*.zip 2>/dev/null | head -1)
    if [[ -z "$asset_file" ]]; then
        fail "No archive downloaded"
        return 1
    fi

    info "Extracting..."
    cd "$temp_dir"
    if [[ "$asset_file" == *.tar.gz || "$asset_file" == *.tgz ]]; then
        tar xzf "$asset_file"
    elif [[ "$asset_file" == *.zip ]]; then
        unzip -q "$asset_file"
    fi

    # Find the binary
    if [[ -n "$archive_binary_path" ]]; then
        # Try exact path first, then search
        if [[ -f "$temp_dir/$archive_binary_path" ]]; then
            binary_file="$temp_dir/$archive_binary_path"
        else
            binary_file=$(find "$temp_dir" -name "$archive_binary_path" -type f | head -1)
        fi
    else
        # Guess: same name as last component of install_path
        local bin_name
        bin_name=$(basename "$install_path")
        binary_file=$(find "$temp_dir" -name "$bin_name" -type f | head -1)
    fi

    if [[ -z "$binary_file" || ! -f "$binary_file" ]]; then
        fail "Binary not found in archive (looked for: ${archive_binary_path:-$(basename "$install_path")})"
        ls -la "$temp_dir"/ >> "$LOG" 2>&1
        return 1
    fi

    chmod +x "$binary_file"

    local install_dir
    install_dir=$(dirname "$install_path")
    if [[ -w "$install_dir" ]]; then
        mv "$binary_file" "$install_path"
    else
        sudo mv "$binary_file" "$install_path"
    fi

    if [[ -n "$post_install" ]]; then
        eval "$post_install" || warn "Post-install hook had errors"
    fi

    xattr -d com.apple.quarantine "$install_path" 2>/dev/null || true

    return 0
}

install_dmg_app() {
    local repo="$1" tag="$2" asset_pattern="$3" install_path="$4" post_install="$5"
    local temp_dir dmg_file volume app_name

    temp_dir=$(mktemp -d)
    trap "rm -rf '$temp_dir'" RETURN

    info "Downloading $asset_pattern from $repo@$tag..."
    if ! gh release download "$tag" --repo "$repo" --pattern "$asset_pattern" -D "$temp_dir" 2>/dev/null; then
        fail "Download failed for $asset_pattern from $repo@$tag"
        return 1
    fi

    dmg_file=$(ls "$temp_dir"/*.dmg 2>/dev/null | head -1)
    if [[ -z "$dmg_file" ]]; then
        fail "No DMG downloaded"
        return 1
    fi

    app_name=$(basename "$install_path")

    info "Mounting DMG..."
    hdiutil attach "$dmg_file" -nobrowse -quiet

    # Find the volume
    volume=$(ls /Volumes/ | grep -i "${app_name%.app}" | head -1)
    if [[ -z "$volume" ]]; then
        # Fallback: find any recently mounted volume
        volume=$(ls -t /Volumes/ | head -1)
    fi

    if [[ -z "$volume" ]]; then
        fail "Could not find mounted volume"
        return 1
    fi

    # Find the .app in the volume
    local source_app
    source_app=$(find "/Volumes/$volume" -maxdepth 1 -name "*.app" -type d | head -1)
    if [[ -z "$source_app" ]]; then
        fail "No .app found in /Volumes/$volume"
        hdiutil detach "/Volumes/$volume" -quiet 2>/dev/null || true
        return 1
    fi

    info "Installing $app_name..."
    rm -rf "$install_path"
    cp -R "$source_app" "$install_path"

    hdiutil detach "/Volumes/$volume" -quiet 2>/dev/null || true

    if [[ -n "$post_install" ]]; then
        eval "$post_install" || warn "Post-install hook had errors"
    fi

    return 0
}

# ─── Update Command ────────────────────────────────────────────────

cmd_update() {
    header "GitHub Tool Updater — Update All"
    log "Starting update run"

    local count updated=0 skipped=0 failed=0 already_current=0
    count=$(tool_count)

    for ((i=0; i<count; i++)); do
        local repo binary_name install_path asset_pattern
        local version_command version_extract tag_prefix tag_filter
        local archive_binary_path post_install tool_type

        repo=$(get_tool_field "$i" "repo")
        binary_name=$(get_tool_field "$i" "binary_name")
        install_path=$(get_tool_field "$i" "install_path")
        asset_pattern=$(get_tool_field "$i" "asset_pattern")
        version_command=$(get_tool_field "$i" "version_command")
        version_extract=$(get_tool_field "$i" "version_extract")
        tag_prefix=$(get_tool_field "$i" "tag_prefix")
        tag_filter=$(get_tool_field "$i" "tag_filter")
        archive_binary_path=$(get_tool_field "$i" "archive_binary_path")
        post_install=$(get_tool_field "$i" "post_install")
        tool_type=$(get_tool_field "$i" "type")

        # Default tag_prefix to "v" if not specified
        if [[ -z "$tag_prefix" ]] && ! jq -e ".tools[$i] | has(\"tag_prefix\")" "$REGISTRY" >/dev/null 2>&1; then
            tag_prefix="v"
        fi

        echo ""
        echo -e "${BOLD}── $binary_name${NC} ($repo)"

        # Get installed version
        local installed_version
        installed_version=$(get_installed_version "$version_command" "$version_extract")
        if [[ -z "$installed_version" ]]; then
            installed_version="not installed"
        fi

        # Get latest version
        local latest_version latest_tag
        latest_version=$(get_latest_version "$repo" "$tag_prefix" "$tag_filter")
        latest_tag=$(get_latest_tag "$repo" "$tag_filter")

        if [[ -z "$latest_version" ]]; then
            fail "Could not fetch latest release for $repo"
            failed=$((failed + 1))
            continue
        fi

        info "Installed: $installed_version | Latest: $latest_version"

        if [[ "$installed_version" == "$latest_version" ]]; then
            ok "Already up to date"
            already_current=$((already_current + 1))
            continue
        fi

        # Perform update
        local result=0
        case "$tool_type" in
            binary)
                install_binary "$repo" "$latest_tag" "$asset_pattern" "$install_path" "$post_install" || result=$?
                ;;
            tarball)
                install_tarball "$repo" "$latest_tag" "$asset_pattern" "$install_path" "$archive_binary_path" "$post_install" || result=$?
                ;;
            dmg-app)
                install_dmg_app "$repo" "$latest_tag" "$asset_pattern" "$install_path" "$post_install" || result=$?
                ;;
            *)
                fail "Unknown type: $tool_type"
                failed=$((failed + 1))
                continue
                ;;
        esac

        if [[ $result -eq 0 ]]; then
            # Verify update
            local new_version
            new_version=$(get_installed_version "$version_command" "$version_extract")
            if [[ -n "$new_version" ]]; then
                ok "Updated $binary_name: $installed_version -> $new_version"
            else
                ok "Installed $binary_name (version check unavailable after update)"
            fi
            updated=$((updated + 1))
        else
            fail "Update failed for $binary_name"
            failed=$((failed + 1))
        fi
    done

    echo ""
    header "Summary"
    echo -e "  ${GREEN}Up to date:${NC}  $already_current"
    echo -e "  ${BLUE}Updated:${NC}     $updated"
    echo -e "  ${RED}Failed:${NC}      $failed"
    echo -e "  Total:       $count"
    log "Update complete: $already_current current, $updated updated, $failed failed"
}

# ─── Status Command ────────────────────────────────────────────────

cmd_status() {
    header "GitHub Tool Updater — Status"

    local count
    count=$(tool_count)

    printf "\n  %-20s %-15s %-15s %s\n" "TOOL" "INSTALLED" "LATEST" "STATUS"
    printf "  %-20s %-15s %-15s %s\n" "────────────────────" "───────────────" "───────────────" "──────────"

    for ((i=0; i<count; i++)); do
        local repo binary_name version_command version_extract tag_prefix tag_filter

        repo=$(get_tool_field "$i" "repo")
        binary_name=$(get_tool_field "$i" "binary_name")
        version_command=$(get_tool_field "$i" "version_command")
        version_extract=$(get_tool_field "$i" "version_extract")
        tag_prefix=$(get_tool_field "$i" "tag_prefix")
        tag_filter=$(get_tool_field "$i" "tag_filter")

        if [[ -z "$tag_prefix" ]] && ! jq -e ".tools[$i] | has(\"tag_prefix\")" "$REGISTRY" >/dev/null 2>&1; then
            tag_prefix="v"
        fi

        local installed_version latest_version status_icon
        installed_version=$(get_installed_version "$version_command" "$version_extract")
        latest_version=$(get_latest_version "$repo" "$tag_prefix" "$tag_filter")

        [[ -z "$installed_version" ]] && installed_version="n/a"
        [[ -z "$latest_version" ]] && latest_version="n/a"

        if [[ "$installed_version" == "$latest_version" ]]; then
            status_icon="${GREEN}current${NC}"
        elif [[ "$installed_version" == "n/a" ]]; then
            status_icon="${YELLOW}not installed${NC}"
        else
            status_icon="${RED}outdated${NC}"
        fi

        printf "  %-20s %-15s %-15s " "$binary_name" "$installed_version" "$latest_version"
        echo -e "$status_icon"
    done
    echo ""
}

# ─── Check Single Tool ─────────────────────────────────────────────

cmd_check() {
    local name="$1"
    local count idx=-1
    count=$(tool_count)

    for ((i=0; i<count; i++)); do
        local bn
        bn=$(get_tool_field "$i" "binary_name")
        if [[ "$bn" == "$name" ]]; then
            idx=$i
            break
        fi
    done

    if [[ $idx -eq -1 ]]; then
        fail "Tool '$name' not found in registry"
        echo "  Registered tools:"
        for ((i=0; i<count; i++)); do
            echo "    - $(get_tool_field "$i" "binary_name")"
        done
        exit 1
    fi

    local repo version_command version_extract tag_prefix tag_filter
    repo=$(get_tool_field "$idx" "repo")
    version_command=$(get_tool_field "$idx" "version_command")
    version_extract=$(get_tool_field "$idx" "version_extract")
    tag_prefix=$(get_tool_field "$idx" "tag_prefix")
    tag_filter=$(get_tool_field "$idx" "tag_filter")

    if [[ -z "$tag_prefix" ]] && ! jq -e ".tools[$idx] | has(\"tag_prefix\")" "$REGISTRY" >/dev/null 2>&1; then
        tag_prefix="v"
    fi

    local installed latest
    installed=$(get_installed_version "$version_command" "$version_extract")
    latest=$(get_latest_version "$repo" "$tag_prefix" "$tag_filter")

    echo "  Tool:      $name"
    echo "  Repo:      $repo"
    echo "  Installed: ${installed:-n/a}"
    echo "  Latest:    ${latest:-n/a}"

    if [[ "$installed" == "$latest" ]]; then
        ok "Up to date"
    elif [[ -z "$installed" ]]; then
        warn "Not installed"
    else
        warn "Update available: $installed -> $latest"
    fi
}

# ─── Scan Command ──────────────────────────────────────────────────

cmd_scan() {
    header "GitHub Tool Updater — Scan for Unregistered Tools"

    # Build list of already-registered binary names and install paths
    local count registered_names="" registered_paths=""
    count=$(tool_count)
    for ((i=0; i<count; i++)); do
        registered_names="$registered_names|$(get_tool_field "$i" "binary_name")"
        registered_paths="$registered_paths|$(get_tool_field "$i" "install_path")"
    done

    # Known non-GitHub tools to skip (managed by brew, npm, pipx, app symlinks, etc.)
    local skip_pattern='^\.(rg-sha256)$|^(canva-mcp|context7|context7\.ts|figma-mcp|notion-mcp|rube|vercel-mcp)$'
    # Symlinks to apps
    skip_pattern="$skip_pattern|^(airbattery|cfgutil|code|codexbar|cursor|docker|docker-compose|docker-credential-.*|hub-tool|kiro|ollama|piactl|pwsh|warp-cli|warp-dex|warp-diag|kubectl|kubectl\.docker)$"
    # Managed by other systems (rustup, pipx, brew, npm, bun, self-updating)
    skip_pattern="$skip_pattern|^(cargo|cargo-clippy|cargo-fmt|cargo-miri|clippy-driver|rls|rust-analyzer|rust-gdb|rust-gdbgui|rust-lldb|rustc|rustdoc|rustfmt|rustup)$"
    skip_pattern="$skip_pattern|^(amp|bunx|claude|droid|droid-.*|lume|ogrep|poetry|pylsp|strix|uv|uvx)$"
    skip_pattern="$skip_pattern|^(mo|mole|youtube-dl|macvdmtool|nmap|ncat|ndiff|nping)$"
    skip_pattern="$skip_pattern|^(topgrade|topgrade_.*)$"
    # Factory MCP wrappers
    skip_pattern="$skip_pattern|^(rg)$"

    local candidates=()

    for dir in "${SCAN_DIRS[@]}"; do
        [[ -d "$dir" ]] || continue
        info "Scanning $dir..."

        for bin in "$dir"/*; do
            [[ -e "$bin" ]] || continue
            [[ -x "$bin" ]] || continue

            local name
            name=$(basename "$bin")

            # Skip if it's a directory
            [[ -f "$bin" || -L "$bin" ]] || continue

            # Skip dotfiles
            [[ "$name" == .* ]] && continue

            # Skip known non-GitHub tools
            if echo "$name" | grep -qE "$skip_pattern" 2>/dev/null; then
                continue
            fi

            # Skip if already registered (by name or path)
            if echo "$registered_names" | grep -qF "|$name" 2>/dev/null; then
                continue
            fi
            local real_path
            real_path=$(readlink -f "$bin" 2>/dev/null || echo "$bin")
            if echo "$registered_paths" | grep -qF "|$real_path" 2>/dev/null; then
                continue
            fi

            # Skip symlinks to pipx, brew, bun, apps
            local target
            target=$(readlink "$bin" 2>/dev/null || echo "")
            if [[ -n "$target" ]]; then
                if echo "$target" | grep -qE "(pipx|/opt/homebrew|\.bun/|/Applications/|\.app/)" 2>/dev/null; then
                    continue
                fi
            fi

            # Try to detect if it's a Mach-O binary or script with version output
            local file_type
            file_type=$(file "$bin" 2>/dev/null || echo "unknown")

            if echo "$file_type" | grep -q "Mach-O" 2>/dev/null; then
                # It's a compiled binary — likely from a GitHub release
                local ver=""
                ver=$("$bin" --version 2>/dev/null | head -1) || \
                ver=$("$bin" -V 2>/dev/null | head -1) || \
                ver=$("$bin" version 2>/dev/null | head -1) || true

                candidates+=("$name|$dir|binary|${ver:-unknown}")
            elif echo "$file_type" | grep -q "script\|text" 2>/dev/null; then
                # Shell script — check if it wraps a GitHub binary
                if grep -qE "(github\.com|gh release)" "$bin" 2>/dev/null; then
                    candidates+=("$name|$dir|script-github|")
                fi
            fi
        done
    done

    if [[ ${#candidates[@]} -eq 0 ]]; then
        ok "No unregistered GitHub tool candidates found"
        echo "  All binaries in scanned directories are either registered or managed by other tools."
        return
    fi

    echo ""
    header "Unregistered Tool Candidates"
    echo "  These Mach-O binaries in your PATH might be from GitHub releases."
    echo "  To add one, run: gh-tool-updater.sh add"
    echo ""

    printf "  %-25s %-35s %-10s %s\n" "NAME" "LOCATION" "TYPE" "VERSION"
    printf "  %-25s %-35s %-10s %s\n" "─────────────────────────" "───────────────────────────────────" "──────────" "────────────────"

    for candidate in "${candidates[@]}"; do
        IFS='|' read -r cname cdir ctype cver <<< "$candidate"
        printf "  %-25s %-35s %-10s %s\n" "$cname" "$cdir" "$ctype" "$cver"
    done
    echo ""
}

# ─── Add Command ────────────────────────────────────────────────────

cmd_add() {
    header "GitHub Tool Updater — Add Tool"

    echo ""
    read -rp "  Binary name (as it appears in PATH): " binary_name
    [[ -z "$binary_name" ]] && { fail "No name provided"; exit 1; }

    # Try to find it
    local bin_path
    bin_path=$(which "$binary_name" 2>/dev/null || true)
    if [[ -z "$bin_path" ]]; then
        echo "  Binary not found in PATH. Enter full path:"
        read -rp "  Install path: " bin_path
    else
        echo "  Found at: $bin_path"
        read -rp "  Install path [$bin_path]: " custom_path
        [[ -n "$custom_path" ]] && bin_path="$custom_path"
    fi

    # Resolve symlinks/wrappers for actual binary location
    if [[ -L "$bin_path" ]]; then
        local real
        real=$(readlink "$bin_path" 2>/dev/null || echo "")
        if [[ -n "$real" ]]; then
            echo "  Symlink target: $real"
        fi
    elif file "$bin_path" 2>/dev/null | grep -q "text" 2>/dev/null; then
        echo "  Note: This is a wrapper script."
        echo "  If it wraps a binary, enter the actual binary path:"
        read -rp "  Real binary path [$bin_path]: " real_path
        [[ -n "$real_path" ]] && bin_path="$real_path"
    fi

    read -rp "  GitHub repo (owner/name): " repo
    [[ -z "$repo" ]] && { fail "No repo provided"; exit 1; }

    # Show available assets
    echo ""
    info "Fetching latest release assets for $repo..."
    local assets_json
    assets_json=$(gh release view --repo "$repo" --json tagName,assets 2>/dev/null) || true

    if [[ -z "$assets_json" ]]; then
        warn "Could not fetch release info. This repo may use a different tag scheme."
        read -rp "  Tag filter (regex for monorepo, empty for standard): " tag_filter
    else
        echo "  Latest tag: $(echo "$assets_json" | jq -r '.tagName')"
        echo "  Assets:"
        echo "$assets_json" | jq -r '.assets[].name' | sed 's/^/    /'
        echo ""
    fi

    read -rp "  Asset pattern for darwin-arm64 (glob): " asset_pattern
    [[ -z "$asset_pattern" ]] && { fail "No asset pattern provided"; exit 1; }

    # Determine type
    echo ""
    echo "  Types: binary (raw executable), tarball (.tar.gz/.zip), dmg-app (.dmg with .app)"
    read -rp "  Type [tarball]: " tool_type
    [[ -z "$tool_type" ]] && tool_type="tarball"

    local archive_binary_path=""
    if [[ "$tool_type" == "tarball" ]]; then
        read -rp "  Binary name inside archive [$binary_name]: " archive_binary_path
        [[ -z "$archive_binary_path" ]] && archive_binary_path="$binary_name"
    fi

    # Version command
    local version_command="$binary_name --version 2>/dev/null"
    read -rp "  Version command [$version_command]: " custom_vc
    [[ -n "$custom_vc" ]] && version_command="$custom_vc"

    local version_extract="awk '{print \$2}'"
    read -rp "  Version extract [$version_extract]: " custom_ve
    [[ -n "$custom_ve" ]] && version_extract="$custom_ve"

    local tag_prefix="v"
    read -rp "  Tag prefix [v]: " custom_tp
    [[ -n "$custom_tp" ]] && tag_prefix="$custom_tp"

    local tag_filter_val="${tag_filter:-}"
    if [[ -z "$tag_filter_val" ]]; then
        read -rp "  Tag filter (regex, empty for standard repos): " tag_filter_val
    fi

    # Build JSON entry
    local new_entry
    new_entry=$(jq -n \
        --arg repo "$repo" \
        --arg binary_name "$binary_name" \
        --arg install_path "$bin_path" \
        --arg asset_pattern "$asset_pattern" \
        --arg version_command "$version_command" \
        --arg version_extract "$version_extract" \
        --arg tag_prefix "$tag_prefix" \
        --arg tag_filter "$tag_filter_val" \
        --arg archive_binary_path "$archive_binary_path" \
        --arg tool_type "$tool_type" \
        '{
            repo: $repo,
            binary_name: $binary_name,
            install_path: $install_path,
            asset_pattern: $asset_pattern,
            version_command: $version_command,
            version_extract: $version_extract,
            type: $tool_type
        }
        + (if $tag_prefix != "v" then {tag_prefix: $tag_prefix} else {} end)
        + (if $tag_filter != "" then {tag_filter: $tag_filter} else {} end)
        + (if $archive_binary_path != "" then {archive_binary_path: $archive_binary_path} else {} end)
        ')

    echo ""
    echo "  New entry:"
    echo "$new_entry" | jq .
    echo ""
    read -rp "  Add to registry? [y/N]: " confirm
    if [[ "$confirm" =~ ^[Yy] ]]; then
        # Add to registry
        local tmp
        tmp=$(mktemp)
        jq ".tools += [$new_entry]" "$REGISTRY" > "$tmp" && mv "$tmp" "$REGISTRY"
        ok "Added $binary_name to registry"
    else
        info "Cancelled"
    fi
}

# ─── Main ───────────────────────────────────────────────────────────

main() {
    local cmd="${1:-help}"

    case "$cmd" in
        update)
            preflight
            cmd_update
            ;;
        status)
            preflight
            cmd_status
            ;;
        scan)
            preflight
            cmd_scan
            ;;
        add)
            preflight
            cmd_add
            ;;
        check)
            preflight
            if [[ -z "${2:-}" ]]; then
                fail "Usage: gh-tool-updater.sh check <tool-name>"
                exit 1
            fi
            cmd_check "$2"
            ;;
        help|--help|-h)
            echo "gh-tool-updater.sh — GitHub Release Tool Manager"
            echo ""
            echo "Commands:"
            echo "  update        Update all registered tools to latest GitHub releases"
            echo "  status        Show installed vs latest versions for all tools"
            echo "  scan          Discover unregistered GitHub tools in binary directories"
            echo "  add           Interactively add a new tool to the registry"
            echo "  check <name>  Check a single tool's status"
            echo "  help          Show this help"
            echo ""
            echo "Registry: $REGISTRY"
            echo "Log:      $LOG"
            ;;
        *)
            fail "Unknown command: $cmd"
            echo "  Run 'gh-tool-updater.sh help' for usage"
            exit 1
            ;;
    esac
}

main "$@"
