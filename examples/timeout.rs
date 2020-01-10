extern crate systemd_boot_conf;

use std::process::exit;
use systemd_boot_conf::SystemdBootConf;

pub fn main() {
    let mut manager = match SystemdBootConf::new("/boot/efi") {
        Ok(manager) => manager,
        Err(why) => {
            eprintln!("failed to get systemd-boot info: {}", why);
            exit(1);
        }
    };

    manager.loader_conf.timeout = Some(10);
    if let Err(why) = manager.overwrite_loader_conf() {
        eprintln!("failed to overwrite systemd-boot loader: {}", why);
        exit(1);
    }

    println!("successfully overwrote loader conf");
    if let Err(why) = manager.load_conf() {
        eprintln!("failed to reload systemd-boot loader conf: {}", why);
        exit(1);
    }

    println!(
        "loader:\n  default: {:?}\n  timeout: {:?}",
        manager.loader_conf.default, manager.loader_conf.timeout
    );
}
