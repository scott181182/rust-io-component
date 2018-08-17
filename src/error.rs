use std::error::Error as StdError;
use std::fmt;

use node_lib::error::Error as NodeLibError;

#[derive(Debug)]
pub enum Error
{
    NodeLib(NodeLibError),
    // TODO: Figure out how to propogate I2C Errors
    I2C(String)
}
impl Error
{
    pub fn from_i2c<E: StdError>(error: E) -> Error
    {
        Error::I2C(format!("{:?}\n{:?}", error.description(), error.cause()))
    }
}

impl fmt::Display for Error
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        use self::Error::*;
        match self
        {
            NodeLib(e) => write!(f, "NodeLib Error: {:?}", e),
            I2C(e)     => write!(f, "I2C Error: {:?}", e)
        }
    }
}

impl StdError for Error
{
    fn description(&self) -> &str
    {
        use self::Error::*;
        match self
        {
            NodeLib(_) => "NodeLib Error",
            I2C(_)     => "I2C Error"
        }
    }

    fn cause(&self) -> Option<&StdError>
    {
        use self::Error::*;
        match self
        {
            NodeLib(err) => Some(err),
            _ => None
        }
    }
}

impl From<NodeLibError> for Error
{
    fn from(error: NodeLibError) -> Error {
        Error::NodeLib(error)
    }
}
