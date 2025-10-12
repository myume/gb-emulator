# gb-emulator
because i've always wanted to make one

## Screenshots

| <img width="640" height="576" alt="image" src="https://github.com/user-attachments/assets/4fd03911-a924-4364-874d-303ab7677344" />   | <img width="640" height="576" alt="image" src="https://github.com/user-attachments/assets/4b265501-50eb-47f7-9de9-1397bff73d4e" /> |
| -------- | ------- |
| <img width="640" height="576" alt="image" src="https://github.com/user-attachments/assets/016ef37f-4c23-4103-84e9-4bd7877f5277" /> | <img width="640" height="576" alt="image" src="https://github.com/user-attachments/assets/75ee1eab-c3ef-4e5d-9133-c3f5d56c3949" /> |

## Controls

| Input    | Key |
| -------- | ------- |
|  Up | W    |
| Down    | S    |
| Left | A     |
| Right    | D    |
| Start    | Enter    |
| Select    | Tab    |
| Toggle Speedup    | Backspace    |

## Codegen
I was too lazy to manually implement each opcode instruction so I just codegen it all in the [build.rs](https://github.com/myume/gb-emulator/blob/main/build/build.rs). I am generating the implementation for each instruction based on [this JSON file](https://github.com/gbdev/gb-opcodes/blob/master/Opcodes.json).

<img width="500" height="500" alt="image" src="https://github.com/user-attachments/assets/cbef9e16-e516-4c81-94ab-eed232097a73" />

## Testing

This passes the Blargg CPU tests and some others. There is no audio support yet so tests relying on APU implementations currently fail.

<img width="640" height="576" alt="image" src="https://github.com/user-attachments/assets/b15c0e47-5cbc-4b0d-ace0-2ed4663dd26e" />

Also passing the [sm83 JSON test suite](https://github.com/SingleStepTests/sm83) (runs in CI).

## Resources that helped me
- https://github.com/jgilchrist/gbemu
- https://github.com/smparsons/retroboy
- https://github.com/mvdnes/rboy
- https://gbdev.io/pandocs
