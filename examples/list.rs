extern crate systemd_boot_conf;

use std::process::exit;
use systemd_boot_conf::SystemdBootConf;

pub fn main() {
    let manager = match SystemdBootConf::new("/boot/efi") {
        Ok(manager) => manager,
        Err(why) => {
            eprintln!("failed to get systemd-boot info: {}", why);
            exit(1);
        }
    };

    println!(
        "loader:\n  default: {:?}\n  timeout: {:?}",
        manager.loader_conf.default, manager.loader_conf.timeout
    );

    for entry in manager.entries {
        println!(
            "  entry: {}\n    title: {}\n    linux: {}\n    initrd: {:?}\n    options: {:?}",
            entry.filename, entry.title, entry.linux, entry.initrd, entry.options
        );
    }
}
