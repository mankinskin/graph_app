# Thesis Build

## Install dependencies

From this directory, run:

```bash
./install-dependencies.sh
```

The script installs a full TeX distribution with LuaLaTeX, Biber, and Libertinus Serif. On Windows, run it from Git Bash or another Bash-compatible shell, then open a new shell if the installation updates `PATH`.

## Build the document

```bash
./build.sh
```

The generated PDF is `out/thesis.pdf`.