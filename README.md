# gb-emulator

because i've always wanted to make one.

my only goal with this was to make pokemon red playable and it seems to be
working, so let's consider it completed.

<img width="340" height="276" alt="image" src="https://github.com/user-attachments/assets/a9e3bff4-9043-4c50-bb41-f8c891b4cec8" />

## Feature Support

- working PPU and CPU
- working Joypad
- serial bus only implemented for blargg test debugging/printing purposes
- basic interrupt handling
- cartridge/gb file parsing
- no audio support

## Usage

To run the emulator, go to the sdl folder and run

```sh
cargo run <path_to_rom>
```

_you may need sdl2 installed locally for this to work_

You can also build it and run it in the same manner.

## Screenshots

| <img width="640" height="576" alt="image" src="https://github.com/user-attachments/assets/4fd03911-a924-4364-874d-303ab7677344" /> | <img width="640" height="576" alt="image" src="https://github.com/user-attachments/assets/4b265501-50eb-47f7-9de9-1397bff73d4e" /> |
| ---------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------- |
| <img width="640" height="576" alt="image" src="https://github.com/user-attachments/assets/016ef37f-4c23-4103-84e9-4bd7877f5277" /> | <img width="640" height="576" alt="image" src="https://github.com/user-attachments/assets/75ee1eab-c3ef-4e5d-9133-c3f5d56c3949" /> |

## Controls

| Input          | Key       |
| -------------- | --------- |
| Up             | W         |
| Down           | S         |
| Left           | A         |
| Right          | D         |
| Start          | Enter     |
| Select         | Tab       |
| Toggle Speedup | Backspace |

## Codegen

I was too lazy to manually implement each opcode instruction individually, so I
just codegen it all in the
[build.rs](https://github.com/myume/gb-emulator/blob/main/build/build.rs).

The implementation for each instruction is generated based on
[this JSON file](https://github.com/gbdev/gb-opcodes/blob/master/Opcodes.json).

## Testing

This passes the Blargg CPU tests and some others. There is no audio support yet
so tests relying on APU implementations currently fail.

<img width="640" height="576" alt="image" src="https://github.com/user-attachments/assets/b15c0e47-5cbc-4b0d-ace0-2ed4663dd26e" />

Also passing the [sm83 JSON test suite](https://github.com/SingleStepTests/sm83)
(runs in CI).

## Resources that helped me

- https://github.com/jgilchrist/gbemu
- https://github.com/smparsons/retroboy
- https://github.com/mvdnes/rboy
- https://gbdev.io/pandocs
