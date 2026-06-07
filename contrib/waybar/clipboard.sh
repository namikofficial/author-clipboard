#!/bin/sh
# Waybar module for author-clipboard status.
#
# Drop this script into your Waybar config as:
#   "custom/clipboard": {
#       "exec": "~/.local/share/author-clipboard/clipboard.sh",
#       "exec-on-event": true,
#       "interval": 30,
#       "signal": 7,
#       "on-click": "author-clipboard-hypr-picker",
#       "on-click-right": "author-clipboard-ctl toggle"
#   }
#
# The script is POSIX sh-compatible (no bashisms).  It requires `jq`
# which is a standard Wayland / Hyprland dependency (Waybar itself
# depends on it).
#
# Outputs a single JSON line for Waybar's custom module format:
#   {"text": "...", "tooltip": "...", "class": "...", "alt": "..."}

CTL="${CTL:-author-clipboard-ctl}"

# Run status command; stderr suppressed so daemon-down is silent.
_status() {
    $CTL status --json 2>/dev/null
}

# Extract one field from the status JSON, or "" if missing.
_jq() {
    _status | jq -r "${1}//\"\""
}

# Convert a decimal number string to an integer for range comparison.
# POSIX sh has no built-in arithmetic on large numbers, so we compare
# as strings.
_gt99() {
    # Returns 0 (success / true) when the number in $1 is > 99.
    case "$1" in
        ''|[!0-9]*) return 1 ;;          # non-numeric or empty -> false
        100|101|102|103|104|105|106|107|108|109|\
        110|111|112|113|114|115|116|117|118|119|\
        1??) return 0 ;;                  # 100+ starts with "1" and 3 digits -> true
        *) return 1 ;;                    # single or double digit -> false
    esac
}

main() {
    running=$(_jq '.running')
    total=$(_jq '.total')
    pinned=$(_jq '.pinned')
    last_type=$(_jq '.last_type')
    last_preview=$(_jq '.last_preview')
    sensitive_last=$(_jq '.sensitive_last')

    # Text label
    if [ -z "$total" ] || [ "$total" -eq 0 ]; then
        text="clipboard: empty"
    elif _gt99 "$total"; then
        text="99+ items"
    else
        text="$total item"
        [ "$total" -ne 1 ] && text="${text}s"
    fi

    # Tooltip and class
    if [ "$running" != "true" ]; then
        tooltip="clipboard: down"
        class="down"
    elif [ "$sensitive_last" = "true" ]; then
        tooltip="Sensitive item"
        [ -n "$total" ] && tooltip="${tooltip}
[${total}] ${last_type} ${pinned} pinned"
        class="running sensitive"
    else
        tooltip="${last_preview}"
        [ -n "$total" ] && tooltip="${tooltip}
[${total}] ${last_type} ${pinned} pinned"
        class="running ${last_type}"
    fi

    alt="${last_type:-text}"

    # Emit JSON for Waybar (jq handles all quoting safely)
    printf '%s\n' "$(
        jq -n \
            --arg text "$text" \
            --arg tooltip "$tooltip" \
            --arg class "$class" \
            --arg alt "$alt" \
            '{text: $text, tooltip: $tooltip, class: $class, alt: $alt}'
    )"
}

main
