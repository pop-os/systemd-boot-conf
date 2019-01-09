# systemd-boot-manager

Rust crate for convenient handling of the systemd-boot loader configuration, as well as the loader
entries that it maintains. This may be used to modify the loader configuration, create new loader
entries, or modify existing ones.

## Examples

Examples may be found in the [examples directory](./examples).

```
# cargo build --examples
# target/debug/examples/list
loader:
  default: Some("Pop_OS-current")
  timeout: None
  entry: Pop_OS-current
    title: Pop!_OS
    linux: /EFI/Pop_OS-ed646eba-b8a3-4c79-8f93-5ee1a25c6ec3/vmlinuz.efi
    initrd: Some("/EFI/Pop_OS-ed646eba-b8a3-4c79-8f93-5ee1a25c6ec3/initrd.img")
    options: ["root=UUID=ed646eba-b8a3-4c79-8f93-5ee1a25c6ec3", "ro", "i8042.nomux", "i8042.reset", "loglevel=0", "quiet", "splash", "systemd.show_status=false", "elevator=bfq"]
  entry: Pop_OS-oldkern
    title: Pop!_OS
    linux: /EFI/Pop_OS-ed646eba-b8a3-4c79-8f93-5ee1a25c6ec3/vmlinuz-previous.efi
    initrd: Some("/EFI/Pop_OS-ed646eba-b8a3-4c79-8f93-5ee1a25c6ec3/initrd.img-previous")
    options: ["root=UUID=ed646eba-b8a3-4c79-8f93-5ee1a25c6ec3", "ro", "i8042.nomux", "i8042.reset", "loglevel=0", "quiet", "splash", "systemd.show_status=false", "elevator=bfq"]
  entry: Recovery-0BE5-B90E
    title: Pop!_OS Recovery
    linux: /EFI/Recovery-0BE5-B90E/vmlinuz.efi
    initrd: Some("/EFI/Recovery-0BE5-B90E/initrd.gz")
    options: ["quiet", "loglevel=0", "systemd.show_status=false", "splash", "boot=casper", "hostname=recovery", "userfullname=Recovery", "username=recovery", "live-media-path=/casper-0BE5-B90E", "noprompt"]
  entry: Pop_OS-xanmod
    title: Pop!_OS
    linux: /EFI/Pop_OS-ed646eba-b8a3-4c79-8f93-5ee1a25c6ec3/vmlinuz-xanmod.efi
    initrd: Some("/EFI/Pop_OS-ed646eba-b8a3-4c79-8f93-5ee1a25c6ec3/initrd-xanmod.img")
    options: ["root=UUID=ed646eba-b8a3-4c79-8f93-5ee1a25c6ec3", "ro", "i8042.nomux", "i8042.reset", "loglevel=0", "quiet", "splash", "systemd.show_status=false", "elevator=bfq"]
```
