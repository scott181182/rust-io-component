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
        .filter_level(LevelFilter::Trace)
        .init();

    let mut io = IoComponent::new("testi2c".to_owned());
    info!("Starting I/O Component...");
    let handles = match io.start() {
        Ok(v) => v,
        Err(e) => {
            return error!("Error starting I/O Component:\n{:?}", e);
        }
    };

    handles.into_iter().for_each(|h| match h.join() {
        Ok(()) => (),
        Err(e) => error!("Error in I/O thread:\n{:?}", e)
    });
}
