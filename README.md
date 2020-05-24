# chip8-emu-rust
> Rust port of my CHIP-8 Emulator (interpreted)

## Prerequisites
* [SDL2](https://www.libsdl.org/download-2.0.php)
* [Rust](https://www.rust-lang.org/)

## Running the emulator
* `cargo run <path-to-ROM-file>`

### Keyboard mappings
* This is the original keypad of the CHIP-8 VM

```
 1 2 3 C 
 4 5 6 D 
 7 8 9 E 
 A 0 B F 
```

* In this emulator, these are mapped to these keys on a keyboard

```
 1 2 3 4 
 Q W E R 
 A S D F 
 Z X C V 
```

### Notes
* I have yet to find the original controls of the games. For now, you must play using trial and error to find the correct keys. Or better, you can research on the original controls (and maybe, link them to me :)). I'll research on this when I have the time.
