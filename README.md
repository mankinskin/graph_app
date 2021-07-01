# Graph App


## Getting started

Start by clicking "Use this template" at https://github.com/emilk/egui_template/ or follow [these instructions](https://docs.github.com/en/free-pro-team@latest/github/creating-cloning-and-archiving-repositories/creating-a-repository-from-a-template).

`src/app.rs` contains a simple example app. This is just to give some inspiration - most of it can be removed if you like.

Make sure you are using the latest version of stable rust.

### Testing locally

`cargo run --release`

On Linux you need to first run `sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev`.

### Compiling for the web

You can compile your app to [WASM](https://en.wikipedia.org/wiki/WebAssembly) and publish it as a web page. For this you need to set up some tools. There are a few simple scripts that help you with this:

``` sh
./setup_web.sh
./build_web.sh
./start_server.sh
open http://127.0.0.1:8080/
```

* `setup_web.sh` installs the tools required to build for web
* `build_web.sh` compiles your code to wasm and puts it in the `docs/` folder (see below)
* `start_server.sh` starts a local HTTP server so you can test before you publish
* Open http://127.0.0.1:8080/ in a web browser to view

The finished web app is found in the `docs/` folder (this is so that you can easily share it with [GitHub Pages](https://docs.github.com/en/free-pro-team@latest/github/working-with-github-pages/configuring-a-publishing-source-for-your-github-pages-site)). It consists of three files:

* `index.html`: A few lines of HTML, CSS and JS that loads your app. **You need to edit this** (once) to replace `egui_template` with the name of your crate!
* `your_crate_bg.wasm`: What the Rust code compiles to.
* `your_crate.js`: Auto-generated binding between Rust and JS.

You can test the template app at <https://emilk.github.io/egui_template/>.
