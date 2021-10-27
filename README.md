# WootingProfileSwitcher
A small tool to automatically switch between the profiles of your Wooting keyboard.

## Installation

### Windows
- Placeholder

### Linux
- ArchLinux [AUR](https://aur.archlinux.org/packages/wootingprofileswitcher-git)
- Other [Build](#linux-1)

## Building

### Windows
- Download [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2019)
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
- Install git, build tools, and x11 headers.
- - Ubuntu/Debian: `apt install git build-essential libx11-dev`
- - ArchLinux/Manjaro: `pacman -S git base-devel`
- Clone the repository.
- - `git clone --recurse-submodules https://github.com/ShayBox/WootingProfileSwitcher.git`
- Navigate to the `linux` directory inside the repository directory.
- - `cd WootingProfileSwitcher/linux`
- Run `make`.

## License
This project is licensed under the MIT License. Read the [License](https://github.com/ShayBox/WootingProfileSwitcher/blob/master/LICENSE) for more information.