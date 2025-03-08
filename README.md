# An Introduction to Game Boy Emulation Using the Rust Programming Language

[https://github.com/aquova/gb-book](https://github.com/aquova/gb-book)

This book is an introductory tutorial for how to develop a Game Boy emulator using the Rust programming language, targeting both desktop computers and web browsers via WebAssembly. It is the follow up to my previous book, [An Introduction to Chip-8 Emulation Development](https://github.com/aquova/chip8-book), also using the Rust language. However, this book is a completely separate project, and you do not need to have completed that tutorial to begin this one. That being said, Game Boy emulation is orders of magnitude more complicated than Chip-8, so I would still strongly familiarize yourself with the concepts described there before tackling this project.

By the end of this book you will have a working emulator for the original Game Boy, capable of playing through many of your favorite games. It should be noted that I have titled this as an *introduction* to Game Boy emulation. Many skilled developers have dedicated their careers to this field, and the emulator you'll have at the end of this project will play games, but won't be terribly accurate, won't support Game Boy Color or Super Game Boy functionality, nor will it have sound. It will be a working emulator, but not top tier one. This guide will prioritize readability and understandability over pure accuracy. That is to say that your emulation journey won't end with the completion of this book, there are still improvements to be made, and I would encourage the curious among you to do so.

While this book is written with the Rust language in mind, very few Rust-specific features are used. If you wish to use a different language, it will be fairly straight-forward to adapt the concepts shown here to a different syntax. The largest exception will be the WebAssembly support, which as of writing, has poor support in most languages.

## About this Book

During the 2020 pandemic lockdown, I decided to create my own Game Boy emulator as a pet project. I found it to be a very rewarding and interesting project, but one that was somewhat difficult to approach. Emulation requires both high and low level knowledge, and it can be difficult in understanding how to combine the two. While there are some really excellent resources out there -- which I will attempt to point you at when relevant -- there are also some notable gaps in easily accessible information when creating an emulator. I recall having to access many different sources for the different subsystems. One for general Game Boy information, one for video rendering, even some resources for related CPU architectures to get information seemingly unavailable elsewhere. This book will attempt to compile all the needed information for basic emulation, although there is certainly more to learn if you wish to dive further in.

Another motivator for writing this book is I find many of the pre-existing references to be a bit too technical for beginners. Many of the most knowledgable Game Boy developers have years of experience in the field, but it can be hard to recall how little a newcomer really knows about the subject. I hope readers find this guide to be approachable at all levels, and that it is able to walk through the material without any preconceptions about what you might already know.

## What you will need

The only requirements to starting this book is having the necessary toolchains installed on your machine. This means you should have the [Rust Programming Language](https://www.rust-lang.org/tools/install) installed and working. Just about any text editor will work for editing Rust, but I would recommend using one with more advanced debugging features. For the desktop application, we will use the [SDL2 library](https://wiki.libsdl.org/SDL2/Installation) to handle the rendering and button inputs, and for the web version we will use [wasm-pack](https://github.com/rustwasm/wasm-pack) to assist with setting up the WebAssembly toolchain. These will need to be correctly installed as well.

Other tools aren't required but will greatly assist with debugging. It's recommended you have a hex editor available for examining Game Boy game files directly. An established emulator is also good to have on hand, to compare their behavior with yours. [Sameboy](https://sameboy.github.io/), [bgb](https://bgb.bircd.org/), or [gearboy](https://github.com/drhelius/Gearboy) are some good examples. You will also need games to test. Legally speaking, I cannot condone the use of proprietary software, but if you happen to come across some, verifying they function correctly would be a big step in developing your emulator. For this tutorial I will use some open source homebrew games as examples, but once we need to verify more complicated behavior, I will also reference some well-known commercial titles.

## Table of Contents

1. [A Refresher on Computer Concepts](book/01-refresher.md)
1. [CPU Specification](book/02-cpu-specs.md)
1. [Project Setup](book/03-project-setup.md)
1. [CPU - Setup](book/04-cpu-setup.md)
1. [CPU - Opcode Setup](book/05-opcode-setup.md)
1. [CPU - Increment/Decrement Instructions](book/06-increment-decrement.md)
1. [CPU - Load Instructions](book/07-load-instructions.md)
1. [CPU - Bitwise Instructions](book/08-bitwise-instructions.md)
1. [CPU - The Stack](book/09-stack.md)
1. [CPU - 0xCB Prefix](book/10-cb-prefix.md)
1. [CPU - Final Misc. Instructions](book/11-final-misc.md)
1. [Memory Bus](book/12-memory-bus.md)
1. [Desktop Frontend Setup](book/13-desktop-setup.md)
1. [Web Frontend Setup](book/14-wasm-setup.md)
1. [Cartridge ROM](book/15-cartridge-rom.md)

With more to come!

## Useful References

I certainly did not perform all the research needed to implement a Game Boy myself. The combined work of many developers over several decades has led to a rich amount of documentation on everything regarding the Game Boy and its operation.

- The Pan Docs, hosted and updated by gbdev.io - https://gbdev.io/pandocs/About.html
- Game Boy: Complete Technical Reference by gekkio - https://gekkio.fi/files/gb-docs/gbctr.pdf
- gbops Opcode Table by izik1 - https://izik1.github.io/gbops/
- Test ROMs by Opus Games - https://opusgames.com/games/GBDev/GBDev.html
- Decoding Z80 Opcodes - http://z80.info/decoding.htm
- WTF is the DAA instruction? - https://ehaskins.com/2018-01-30%20Z80%20DAA/
- Game Boy Emulator Development Guide - https://hacktix.github.io/GBEDG/
- Detailed description of MBC types - https://gbdev.gg8.se/wiki/articles/Memory_Bank_Controllers
