# WootingProfileSwitcher
A small tool to automatically switch between the profiles of your Wooting keyboard.

## Requirements
### Windows:
- [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2019)

### Linux:
- gcc
- make
- pkg-config
- libx11

## Building
### Windows
- Open the `x64 Native Tools Command Prompt for VS 2019` or `x86 Native Tools Command Prompt for VS 2019` prompt depending on what architecture you plan to build.
- Navigate to the `windows` folder inside the repository files.
#### 32-Bit
```
nmake release32
```
#### 64-Bit
```
nmake release64
```
#### Debug
To build debug builds just replace `release` with `debug` in the commands you run.

### Linux
- Open a shell of your choice.
- Navigate to the `linux` folder inside the repository files.
- Run `make`.

## License
This project is licensed under the MIT License. Read the [License](https://github.com/ShayBox/WootingProfileSwitcher/blob/master/LICENSE) for more information.