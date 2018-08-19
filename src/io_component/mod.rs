pub mod i2c;

use error::Error;

use node_lib::node::Node;

use std::sync::Arc;
use std::thread::JoinHandle;



pub struct IoComponent
{
    i2c_bridge: i2c::I2CBridge
}
impl IoComponent
{
    pub fn new(name: String) -> IoComponent
    {
        let node: Arc<Node> = Node::new(name, Some("Orchestrator;10.3.33.240".parse().unwrap())).unwrap().into();
        IoComponent{ i2c_bridge: i2c::I2CBridge::new(node) }
    }

    pub fn start(&mut self) -> Result<Vec<JoinHandle<()>>, Error>
    {
        self.i2c_bridge.configure()
    }
}
