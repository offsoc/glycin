use crate::error::ConversionTooLargerError;

pub trait SafeConversion:
    TryInto<usize> + TryInto<i32> + TryInto<u32> + TryInto<i64> + TryInto<u64>
{
    fn try_usize(self) -> Result<usize, ConversionTooLargerError> {
        self.try_into().map_err(|_| ConversionTooLargerError)
    }

    fn try_i32(self) -> Result<i32, ConversionTooLargerError> {
        self.try_into().map_err(|_| ConversionTooLargerError)
    }

    fn try_u32(self) -> Result<u32, ConversionTooLargerError> {
        self.try_into().map_err(|_| ConversionTooLargerError)
    }

    fn try_i64(self) -> Result<i64, ConversionTooLargerError> {
        self.try_into().map_err(|_| ConversionTooLargerError)
    }

    fn try_u64(self) -> Result<u64, ConversionTooLargerError> {
        self.try_into().map_err(|_| ConversionTooLargerError)
    }
}

impl SafeConversion for usize {}
impl SafeConversion for u32 {}
impl SafeConversion for i32 {}
