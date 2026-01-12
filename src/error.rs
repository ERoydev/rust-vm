use derive_more::{Display, From};

pub type Result<T> = core::result::Result<T, VMError>;

#[derive(Debug, Display, From)]
#[display("{self:?}")]
pub enum VMError {
    // memory
    OutOfBounds,

    // register
    UnknownRegister,

    // vm
    Halted,
    MemoryReadError,
    OpcodeDoesNotExist,

    // bus
    AddInstructionFail,
    CopyInstructionFail,

    // -- Externals
    #[from]
    Io(std::io::Error),
}

impl VMError {
    pub fn message(&self) -> &'static str {
        match self {
            VMError::UnknownRegister => "Unknown Register",
            VMError::OutOfBounds => "Memory access is out of bounds",
            VMError::Halted => "Cannot use a Halted machine",
            VMError::MemoryReadError => "Memory read failed",
            _ => "Else",
        }
    }

    // Placeholder
    // pub fn custom_from_err(err: impl std::error::Error) -> Self {
    //     Self::Custom(err.to_string())
    // }
    // pub fn custom(val: impl Into<String>) -> Self {
    //     Self::Custom(val.into())
    // }
}

impl std::error::Error for VMError {}
