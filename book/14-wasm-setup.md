# XIV. WebAssembly Frontend Setup

[*Return to Index*](../README.md)

[*Previous Chapter*](13-desktop-setup.md)

With our `desktop` frontend in a good place, we shall turn our attention to the `wasm` frontend and complete many of the same procedures. If you are skipping to here without reading the previous chapter, I would recommend you go back and at least skim it, as it provides the justification for some of the decisions we're about to make, as well as performs some changes to the `core` backend that we will again rely upon here.

We'll begin by adding in some new dependencies to our `wasm/Cargo.toml` configuration file. We already have the entry pointing us to the `core` backend module, but we will need to include three third party libraries. The first two, `js-sys` and `web-sys` allow Rust modules access to JavaScript and HTML DOM elements respectively, both of which we will need when interfacing with the webpage. The third, `wasm-bindgen`, provides support for interacting between Rust and JavaScript more easily. I've set them to be the most up-to-date versions at the time of writing, but you should cross-reference with the online versions to see if any new changes are available -- https://crates.io/.

```toml
# In wasm/Cargo.toml

[dependencies]
gb_core = { path = "../core" }
js-sys = "0.3.67"
wasm-bindgen = "0.2.90"

[dependencies.web-sys]
version = "0.3.67"
features = [
    "CanvasRenderingContext2d",
    "Document",
    "Element",
    "HtmlCanvasElement",
    "ImageData",
    "KeyboardEvent",
    "Window",
]

[lib]
crate-type = ["cdylib"]
```

You'll notice the syntax for `web-sys` is rather unusual. There are a lot of HTML elements in the world, but we will need very few of them for our project. Rather than import them all, I've only called out the specific ones that we will use; a list that hopefully seems pretty logical. We also set the `crate-type` of this module to be `"cdylib"`. This is a compatibility feature that specifies that this module is meant to be utilized by another programming language -- in this case JavaScript.

With this in place, we can turn our attention to the WebAssembly implementation. Let's take a moment and first describe what we're actually going to create. The user will directly interact with a HTML webpage with a `canvas` element within it. This `canvas` will display the Game Boy screen data provided up from the backend, and the page will provide the loaded ROM data and keyboard presses down to it. Where the WebAssembly module fits in is it provides public API functions for JavaScript to utilize that accesses functions within `core`. Think of it (and an accompanying auto-generated JavaScript file you'll see in a moment) as the connective "glue" between the webpage and the `core` backend. It should be noted that the compiled `.wasm` file will be a single binary which includes both all of `core` and `wasm/src/lib.rs` together as one.

In order to get a better sense of how this will fit together, let's create the webpage first. This page won't be anything special (or be good looking), and should be simple to anyone with even an introductory web background. We'll create this folder space outside of our `wasm` directory. Back up at the root of the project, create a fourth directory called `html` with an `index.html` file inside it.

```html
<!DOCTYPE html>
<html>
    <head>
        <title>Game Boy Emulator</title>
        <meta charset="utf-8">
        <style>
            html {
                text-align: center;
                font-family: "Arial", "Helvetica", sans-serif;
                max-width: 1000px;
                margin: 0 auto;
            }

            canvas {
                padding-left: 0;
                padding-right: 0;
                margin-left: auto;
                margin-right: auto;
            }
        </style>
    </head>
    <body>
        <h1>My Game Boy Emulator</h1>
        <label for="fileinput">Select a GB game: </label>
        <input type="file" id="fileinput" accept=".gb,.dmg" autocomplete="off"/>
        <br/><br/>
        <canvas id="canvas" width="160px" height="144px">If you can see this, then your browser doesn't support HTML5 and is old.</canvas>
    </body>
    <script type="module" src="index.js"></script>
</html>
```

Beautiful. This is about as barebones as they come. Feel free to edit the CSS styling as you wish, I'm a terrible graphic designer. The page includes only some header text, the `canvas` element -- which defaults to the familiar 160x144 Game Boy resolution -- and a button which will prompt the user to select their Game Boy ROM. Just like with the `desktop` frontend, we don't do any verification to determine if the selected file is a valid Game Boy game, but here we at least limit the selection to the most well-used file extensions (I have very rarely seen other extensions used for GB titles, such as `.bin`, or `.cgb` and `.gbc` for Color titles. You're free to edit the list or remove the `accept` limitation altogether). At the end of the page, an `index.js` file is referenced. Let's create it and populate it now.

```javascript
// In html/index.js

const SCALE = 3
const WIDTH = 160
const HEIGHT = 144

let canvas = document.getElementById("canvas")
canvas.width = WIDTH * SCALE
canvas.height = HEIGHT * SCALE

let ctx = canvas.getContext("2d")
ctx.fillStyle = "#FFFFFF"
ctx.fillRect(0, 0, canvas.width, canvas.height)

async function run() {
    document.getElementById("fileinput").addEventListener("change", function (e) {
        let file = e.target.files[0]
        if (!file) {
            alert("Failed to read file")
            return
        }

        let fr = new FileReader()
        fr.onload = function (fre) {
            let buffer = fre.result
            const rom = new Uint8Array(buffer)
            // TODO: Load ROM
        }

        fr.readAsArrayBuffer(file)
    }, false)
}

run().catch(console.error)
```

We'll start with a few things present. First, the script grabs the `canvas` UI element, scales its size, and colors it completely white to start. Next is the `run` function which includes an event handler for the file select button. When it's clicked, the page attempts to read the specified file in as a `Uint8Array` buffer, but right now has no where to put it. At this point, if you've been following along with the entire tutorial your project structure should look like this.

```
.
├── core
│   ├── Cargo.toml
│   └── src
│       ├── bus.rs
│       ├── cpu
│       │   ├── mod.rs
│       │   └── opcodes.rs
│       ├── lib.rs
│       └── utils.rs
├── desktop
│   ├── Cargo.toml
│   └── src
│       └── main.rs
├── wasm
│   ├── Cargo.toml
│   └── src
│       └── lib.rs
└── html
    ├── index.html
    └── index.js
```

It's at this point that we're stuck with regards to what we can do on the `html` side. We now must return to the `wasm` module and implement the functions that this JavaScript program will need to continue. These are the same that we needed in the `desktop` frontend -- a constructor for the `Cpu` object and a function to pass in the ROM data. This is where `wasm_bindgen` comes in. It has declarators for functions and constructors so that they can properly be interfaced with. In `wasm/src/lib.rs`, we're going to create an object with two member functions -- our `core` CPU object and a reference to the HTML `canvas` element, which will be useful to have later.

```rust
// In wasm/src/lib.rs

use gb_core::cpu::Cpu;

use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

#[wasm_bindgen]
pub struct GB {
    cpu: Cpu,
    ctx: CanvasRenderingContext2d,
}

#[wasm_bindgen]
impl GB {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<GB, JsValue> {
        let cpu = Cpu::new();

        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id("canvas").unwrap();
        let canvas: HtmlCanvasElement = canvas.dyn_into::<HtmlCanvasElement>()
            .map_err(|_| ())
            .unwrap();

        let ctx = canvas.get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();

        let gb = GB { cpu, ctx };
        Ok(gb)
    }

    #[wasm_bindgen]
    pub fn load_rom(&mut self, data: Uint8Array) {
        let mut rom: Vec<u8> = Vec::new();

        for i in 0..data.byte_length() {
            rom.push(data.get_index(i));
        }
        self.cpu.load_rom(&rom);
    }
}
```

With the exception of the `wasm_bindgen` everywhere, this should look pretty familiar. The creation of the `cpu` field only requires us to call `Cpu::new()`, no surprises there. Getting the canvas `ctx` from the webpage is very similar to the steps one would take in JavaScript, but with a lot more `unwrap` at every step. Once we have both of them we bundle them in a `Result`, which is required for constructors. The `load_rom` function also looks very similar to what we did in the `desktop` module. We receive the data as a `Uint8Array` and will need to iterate across it to convert each byte into a Rust-style `u8`, at which point we can pass it into `core` via the `load_rom` function we added in the previous chapter.

## Compilation

Compiling a WebAssembly module requires a bit of setup beyond what you'll typically get out of the box with a Rust install. Rather than go through the trouble of setting up all the toolchains ourselves, I'm going to rely on a third party tool known as `wasm-pack` to handle this for us. I personally like to rely on as few mysterious tools as I can, but given it's from the same group that develops `wasm-bindgen`, I'm willing to make an exception. You can install `wasm-pack` via cargo if you haven't yet.

```
$ cargo install wasm-pack
```

Once it's finished installing, enter the `wasm` directory and build our project via the following command.

```
$ wasm-pack build --target=web
```

It will complain a bit about some unused items, but should complete successfully. Once it does, there are two newly generated files we are interested in -- `pkg/wasm_bg.wasm` and `pkg/wasm.js`. `pkg/wasm_bg.wasm` is the WebAssembly binary I've been promising, containing both the `wasm/src/lib.rs` file we just finished editing as well as the entirety of `core`, together in a single binary file. `pkg/wasm.js` is a surprisingly short and surprisingly readable helper file, created to assist in loading and setting up the API we defined in the `wasm` module. Both of these files need to be copied over into the `html` directory, and will need to be replaced each time we recompile `wasm`. With them in place, all that remains is to access them in `html/index.js`.

```javascript
// In html/index.js

import init, * as wasm from "./wasm.js"

// Unchanged code omitted

async function run() {
    await init()
    let gb = new wasm.GB()

    document.getElementById("fileinput").addEventListener("change", function (e) {
        let file = e.target.files[0]
        if (!file) {
            alert("Failed to read file")
            return
        }

        let fr = new FileReader()
        fr.onload = function () {
            let buffer = fr.result
            const rom = new Uint8Array(buffer)
            gb.load_rom(rom)
        }

        fr.readAsArrayBuffer(file)
    }, false)
}

run().catch(console.error)
```

The `wasm_bg.wasm` file is loaded by the generated `wasm.js` file once its `init` function is called, which we need to do before anything else. After that, we have access to the functions we defined in `wasm/src/lib.rs`. These are our constructor, which we save into the `gb` variable, and the `load_rom` function, which we use to send our `Uint8Array` of ROM data. If you start up a local webserver and load this webpage, you'll notice... not too much. Unlike the `desktop` frontend where a window appears, the `html` page doesn't really do much since there's no emulation happening. We'll work to resolve that soon, but at this point both our frontends are in the position to read in a ROM data file and pass it into the `core` backend. Our next job is to return to `core` and create something to receive it.

[*Next Chapter*](15-cartridge-rom.md)
