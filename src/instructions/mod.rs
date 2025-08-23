use pinocchio::program_error::ProgramError;

pub mod make;
pub mod refund;
pub mod take;

pub use make::*;
pub use refund::*;
pub use take::*;

#[repr(u8)]
pub enum EscrowInstruction {
    MakeOffer,
    TakeOffer,
    Refund,
}

impl TryFrom<&u8> for EscrowInstruction {
    type Error = ProgramError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match *value {
            0 => Ok(EscrowInstruction::MakeOffer),
            1 => Ok(EscrowInstruction::TakeOffer),
            2 => Ok(EscrowInstruction::Refund),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}
