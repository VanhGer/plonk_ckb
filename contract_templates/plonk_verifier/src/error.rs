/// Defines a custom `Error` enum and implements a conversion from `SysError`.
///
/// The `Error` enum represents various error conditions that can occur in the program,
/// each with a distinct error code for easy identification.
///
/// # Variants
/// - `IndexOutOfBound`: Error code 1, indicates an index is out of bounds.
/// - `ItemMissing`: Error code 2, indicates a missing item.
/// - `LengthNotEnough`: Error code 3, indicates a length is not sufficient.
/// - `Encoding`: Error code 4, indicates an encoding error.
/// - `Verify`: Error code 5, indicates a verification error.
/// - `LoadingCell`: Error code 6, indicates an error while loading a cell.
///
/// # Note
/// The implementation includes a panic for unknown system errors, ensuring that all handled
/// system errors are explicitly mapped to a corresponding custom error variant.
use ckb_std::error::SysError;

#[repr(i8)]
pub enum Error {
    IndexOutOfBound = 1,
    ItemMissing,
    LengthNotEnough,
    Encoding,
    Verify,
    LoadingCell,
}

impl From<SysError> for Error {
    fn from(err: SysError) -> Self {
        use SysError::*;
        match err {
            IndexOutOfBound => Self::IndexOutOfBound,
            ItemMissing => Self::ItemMissing,
            LengthNotEnough(_) => Self::LengthNotEnough,
            Encoding => Self::Encoding,
            Unknown(err_code) => panic!("unexpected sys error {}", err_code),
        }
    }
}
