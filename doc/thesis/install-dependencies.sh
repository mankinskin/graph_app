#!/usr/bin/env bash

set -euo pipefail

if command -v lualatex >/dev/null 2>&1 && command -v biber >/dev/null 2>&1; then
    case "$(uname -s)" in
        MINGW*|MSYS*|CYGWIN*)
            MIKTEX_BIN="$(dirname "$(command -v lualatex)")"
            "$MIKTEX_BIN/mpm.exe" --install=libertinus-fonts
            ;;
    esac
    echo "LuaLaTeX and Biber are already installed."
    exit 0
fi

case "$(uname -s)" in
    Linux)
        if command -v apt-get >/dev/null 2>&1; then
            sudo apt-get update
            sudo apt-get install -y texlive-full biber
        elif command -v dnf >/dev/null 2>&1; then
            sudo dnf install -y texlive-scheme-full biber
        elif command -v pacman >/dev/null 2>&1; then
            sudo pacman -Sy --needed texlive-meta biber
        else
            echo "Unsupported Linux package manager. Install a full TeX Live distribution and Biber." >&2
            exit 1
        fi
        ;;
    Darwin)
        if ! command -v brew >/dev/null 2>&1; then
            echo "Homebrew is required. Install it from https://brew.sh, then rerun this script." >&2
            exit 1
        fi
        brew install --cask mactex-no-gui
        brew install biber
        ;;
    MINGW*|MSYS*|CYGWIN*)
        if ! command -v winget >/dev/null 2>&1; then
            echo "winget is required. Install MiKTeX manually, then ensure lualatex and biber are on PATH." >&2
            exit 1
        fi
        winget install --exact --id MiKTeX.MiKTeX --source winget --accept-package-agreements --accept-source-agreements
        MIKTEX_BIN="$(cygpath "$LOCALAPPDATA")/Programs/MiKTeX/miktex/bin/x64"
        "$MIKTEX_BIN/mpm.exe" --install=libertinus-fonts
        ;;
    *)
        echo "Unsupported operating system. Install a full TeX Live or MiKTeX distribution and Biber." >&2
        exit 1
        ;;
esac

echo "Installation complete. Open a new shell if lualatex or biber are not yet on PATH."