

// TODO: Move this to an MWAVE messages package
#[derive(Serialize, Deserialize, Debug)]
pub struct I2CDevice
{
    pub bus: u8,
    pub address: u8,
    pub frequency: u16,
    pub device: String,
    pub option_keys: Vec<String>,
    pub option_values: Vec<String>,
    pub topics: [String; 16]
}
