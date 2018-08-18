extern crate i2cdev_lib;
extern crate messages;
extern crate node_lib;

mod error;
mod io_component;

use io_component::IoComponent;

fn main()
{
    let mut io = IoComponent::new("RelayComponent".to_owned());
    io.start();
}
