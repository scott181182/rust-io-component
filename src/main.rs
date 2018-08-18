#[macro_use] extern crate log;
extern crate env_logger;

extern crate i2cdev_lib;
extern crate messages;
extern crate node_lib;

mod error;
mod io_component;

use io_component::IoComponent;

use env_logger::{ Builder as LogBuilder };
use log::LevelFilter;

fn main()
{
    LogBuilder::new()
        .default_format()
        .default_format_module_path(false)
        .default_format_timestamp(false)
        .filter_level(LevelFilter::Info)
        .init();

    let mut io = IoComponent::new("RelayComponent".to_owned());
    info!("Starting I/O Component...");
    io.start();
}
