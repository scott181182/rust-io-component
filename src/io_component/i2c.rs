use error::Error;

use std::sync::{ Arc, Mutex, MutexGuard, LockResult };

use messages::msgs;
use node_lib::node::Node;



// Need to be aware of what devices to include dependent on OS

#[cfg(not(target_os = "linux"))]
use i2cdev_lib::prelude::MockI2CDevice;
#[cfg(not(target_os = "linux"))]
type I2CDeviceType = MockI2CDevice;

#[cfg(target_os = "linux")]
use i2cdev_lib::prelude::LinuxI2CDevice;
#[cfg(target_os = "linux")]
type I2CDeviceType = LinuxI2CDevice;




const I2CCONFIG_SERVICE: &str = "FetchIoConfig";

pub struct I2CBridge
{
    node: Arc<Node>,

    pca9555_vec: Vec<pca9555::Bridge>
}
impl I2CBridge
{
    pub fn new(node: Arc<Node>) -> I2CBridge
    {
        I2CBridge{ node, pca9555_vec: vec![  ] }
    }
    pub fn configure(&mut self) -> Result<(), Error>
    {
        let devices: Vec<msgs::I2CDevice> = self.node.call_service(I2CCONFIG_SERVICE.to_owned(), &self.node.get_name().to_owned())
            .map_err(Error::from)?;

        for device in devices
        {
            #[cfg(target_os = "linux")]
            let interface = LinuxI2CDevice::new(format!("/dev/i2c-{}", device.bus), device.address as u16)
                .map_err(Error::from_i2c)?;
            #[cfg(not(target_os = "linux"))]
            let interface = MockI2CDevice::new();

            if device.device == "pca9555" {
                let mut bridge = pca9555::Bridge::new(interface, device);
                bridge.init(&self.node)?;
                self.pca9555_vec.push(bridge);
            }
        }

        Ok(())
    }
}



pub struct I2Cell<T>
{
    device_lock: Arc<Mutex<T>>
}
impl <T> I2Cell<T>
{
    pub fn new(device: T) -> I2Cell<T>
    {
        I2Cell{ device_lock: Arc::new(Mutex::new(device)) }
    }
    pub fn lock(&self) -> LockResult<MutexGuard<T>>
    {
        self.device_lock.lock()
    }
}
impl <T> Clone for I2Cell<T>
{
    fn clone(&self) -> I2Cell<T> { I2Cell{ device_lock: self.device_lock.clone() } }
}



mod pca9555
{
    use super::{ I2Cell, I2CDeviceType };
    use error::Error;

    use std::sync::{ Arc };
    use std::sync::atomic::{ AtomicBool, Ordering };
    use std::thread;

    use i2cdev_lib::pca9555::PCA9555;
    use messages::msgs;
    use node_lib::node::{ Node, Publisher, MultiSubscriber };

    pub struct Bridge
    {
        device: I2Cell<PCA9555<I2CDeviceType>>,
        frequency: u16,
        config_mask: u16,
        topics: [String; 16],
        running: Arc<AtomicBool>
    }
    impl Bridge
    {
        pub fn new(interface: I2CDeviceType, config: msgs::I2CDevice) -> Self
        {
            let config_mask: u16 = if config.options.contains_key("mode") {
                if config.options["mode"] == "output" { 0x0000 } else { 0xffff }
            } else {
                u16::from_str_radix(&config.options["mask"], 16)
                    .map_err(|e| error!("Error parsing config_mask for PCA9555:\n{:?}", e))
                    .unwrap_or(0)
            };

            Bridge {
                device: I2Cell::new(PCA9555::from(interface)),
                frequency: config.frequency,
                config_mask,
                topics: config.topics,
                running: Arc::new(true.into())
            }
        }

        pub fn init(&mut self, node: &Node) -> Result<(), Error>
        {
            {
                let mut lock = self.device.lock().unwrap();
                lock.write_config(self.config_mask).map_err(Error::from_i2c)?;
                lock.write_output(0).map_err(Error::from_i2c)?;
            }

            let (outputs, inputs): (Vec<(usize, String)>, Vec<(usize, String)>) = self.topics.iter().cloned().enumerate()
                .filter(|(_, topic)| !topic.is_empty())
                .partition(|(pin, _)| self.config_mask & (1u16 << pin) == 0);

            if !outputs.is_empty()
            {
                let output_topics = outputs.iter()
                    .map(|(_, topic)| topic)
                    .cloned().collect();
                let sub = node.create_multisubscriber::<bool>(output_topics)?;
                self.spawn_listener(outputs, sub);
            }
            if !inputs.is_empty()
            {
                let pubs = inputs.into_iter()
                    .map(|(index, topic)| (index, node.create_publisher(topic)))
                    .collect();
                self.spawn_poller(pubs);
            }

            Ok(())
        }

        pub fn spawn_listener(&self, topics: Vec<(usize, String)>, sub: MultiSubscriber<bool>) -> thread::JoinHandle<()>
        {
            let device = self.device.clone();

            thread::spawn(move || {
                while let Ok(message) = sub.recv()
                {
                    if let Some((pin, _)) = topics.iter()
                        .find(|(_, topic)| topic == &message.topic)
                    {
                        let _ = device.lock().unwrap().write_output_pin(*pin as u8, message.payload)
                            .map_err(|e| error!("Error writing to PCA9555;\n{:?}", e));
                    }

                }
            })
        }
        pub fn spawn_poller(&self, topics: Vec<(usize, Publisher<bool>)>) -> thread::JoinHandle<()>
        {
            use std::time::Duration;

            let period = Duration::from_millis(f64::round(1000f64 / self.frequency as f64) as u64);
            let mut prev = !self.device.lock().unwrap().read_input()
                .expect("Error polling PCA9555");
            let device = self.device.clone();
            let control = self.running.clone();

            // TODO: Handle unwraps more gracefully
            thread::spawn(move || {
                while control.load(Ordering::Relaxed)
                {
                    let result = match device.lock().unwrap().read_input() {
                        Ok(r) => r,
                        Err(e) => {
                            error!("Error polling PCA9555:\n{:?}", e);
                            continue;
                        }
                    };

                    topics.iter()
                        .filter(|(pin, _)| (result ^ prev) & (1u16 << pin) > 0)
                        .for_each(|(pin, publisher)|
                            publisher.publish(&(result & (1u16 << pin) > 0)).unwrap());

                    prev = result;
                    thread::sleep(period);
                }
            })
        }
    }
}
