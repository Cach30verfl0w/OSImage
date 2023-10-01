<div align="center">
  
# `OSImage`

![GitHub](https://img.shields.io/github/license/Cach30verfl0w/OSImage) ![GitHub issues](https://img.shields.io/github/issues/Cach30verfl0w/OSImage) ![GitHub code size in bytes](https://img.shields.io/github/languages/code-size/Cach30verfl0w/OSImage) ![GitHub commit activity (branch)](https://img.shields.io/github/commit-activity/y/Cach30verfl0w/OSImage) ![GitHub last commit (branch)](https://img.shields.io/github/last-commit/Cach30verfl0w/OSImage/main)
![GitHub pull requests](https://img.shields.io/github/issues-pr/Cach30verfl0w/OSImage)

Command-Line Tool to generate image files and run them in QEMU for Rust Operating Systems. Subproject of [`OverflowOS`](https://github.com/Cach30verfl0w/OverflowOS)

</div>

## Commands
Here is a list with the commands for this tool:

- `build-image` - Build the ISO image from the specified workspace/project
   - `image-file` - The name of the image file that should be built by this tool (default: image.img)
   - `iso-file` - The name of the ISO file file that should be built by this tool (default: image.iso)
   - `block-size` - Size of the sectors in the image file (default: 512 bytes)
   - `block-count` - Count of sectors in the image file (default: 93750 sectors)
- `run-qemu` - Run the built image with OVMF in QEMU
