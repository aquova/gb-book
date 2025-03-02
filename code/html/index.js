import init, * as wasm from "./wasm.js"

const SCALE = 3
const WIDTH = 160
const HEIGHT = 144

let canvas = document.getElementById("canvas")
canvas.width = WIDTH * SCALE
canvas.height = HEIGHT * SCALE

let anim_frame = 0

let ctx = canvas.getContext("2d")
ctx.fillStyle = "#FFFFFF"
ctx.fillRect(0, 0, canvas.width, canvas.height)

async function run() {
    await init()
    let gb = new wasm.GB()

    document.addEventListener("keydown", function(e) {
        gb.press_button(e, true)
    })

    document.addEventListener("keyup", function(e) {
        gb.press_button(e, false)
    })

    document.getElementById("fileinput").addEventListener("change", function (e) {
        // Stop previous game from rendering, if one exists
        if (anim_frame != 0) {
            window.cancelAnimationFrame(anim_frame)
        }

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
            let title = gb.get_title()
            document.title = title

            mainloop(gb)
        }

        fr.readAsArrayBuffer(file)
    }, false)
}

function mainloop(gb) {
    while (true) {
        let draw_time = gb.tick()
        if (draw_time) {
            gb.draw_screen()
            if (SCALE != 1) {
                let ctx = canvas.getContext('2d')
                ctx.imageSmoothingEnabled = false
                ctx.drawImage(canvas, 0, 0, WIDTH, HEIGHT, 0, 0, canvas.width, canvas.height)
            }

            anim_frame = window.requestAnimationFrame(() => {
                mainloop(gb)
            })
            return
        }
    }
}

run().catch(console.error)
