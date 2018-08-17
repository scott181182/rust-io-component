#[macro_use]
extern crate serde_derive;

extern crate i2cdev_lib;
extern crate node_lib;
extern crate serde;

mod util;
mod error;
mod msg;
mod io_component;

use io_component::IoComponent;

fn main()
{
    let mut io = IoComponent::new("RelayComponent".to_owned());
    io.start();
}
