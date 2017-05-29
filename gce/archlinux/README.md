# Build ArchLinux image for Google Compute Engine from scratch

## Usage

```sh
# Launch new instance and start building ArchLinux root
./create-image.sh
# Wait until startup-script.sh finishes
ssh archlinux-builder-${id}
# Create image and delete resources
./create-image-next-${id}.sh
```
