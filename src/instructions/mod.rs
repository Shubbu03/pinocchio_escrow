use pinocchio::program_error::ProgramError;

pub mod make_offer;
pub mod refund;
pub mod take_offer;

pub use make_offer::*;
pub use refund::*;
pub use take_offer::*;

#[repr(u8)]
pub enum ProgramInstruction {
    MakeOffer,
    TakeOffer,
    Refund,
}

impl TryFrom<&u8> for ProgramInstruction {
    type Error = ProgramError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match *value {
            0 => Ok(ProgramInstruction::MakeOffer),
            1 => Ok(ProgramInstruction::TakeOffer),
            2 => Ok(ProgramInstruction::Refund),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}
