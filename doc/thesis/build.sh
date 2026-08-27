#!/usr/bin/env bash

set -euo pipefail

cd "$(dirname "$0")"

for command in lualatex biber; do
    if ! command -v "$command" >/dev/null 2>&1; then
        echo "Missing $command. Run ./install-dependencies.sh first." >&2
        exit 1
    fi
done

mkdir -p out

lualatex -interaction=nonstopmode -halt-on-error -jobname=thesis -output-directory=out main.tex
biber --input-directory out --output-directory out thesis
lualatex -interaction=nonstopmode -halt-on-error -jobname=thesis -output-directory=out main.tex
lualatex -interaction=nonstopmode -halt-on-error -jobname=thesis -output-directory=out main.tex

echo "Built out/thesis.pdf"