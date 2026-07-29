use core::{fmt, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

use crate::error::GuardianError;

/// Solana address boundary type independent of crate-specific public-key types.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Address32(pub [u8; 32]);

impl Address32 {
    pub const ZERO: Self = Self([0; 32]);

    #[must_use]
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl fmt::Display for Address32 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&bs58::encode(self.0).into_string())
    }
}

impl FromStr for Address32 {
    type Err = GuardianError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let bytes = bs58::decode(value)
            .into_vec()
            .map_err(|_| GuardianError::invalid_input("address is not valid base58"))?;
        let array = <[u8; 32]>::try_from(bytes.as_slice())
            .map_err(|_| GuardianError::invalid_input("address must decode to 32 bytes"))?;
        Ok(Self(array))
    }
}

impl Serialize for Address32 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Address32 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        value.parse().map_err(de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::Address32;

    #[test]
    fn round_trips_base58() {
        let address = Address32::new([7; 32]);
        assert_eq!(address.to_string().parse(), Ok(address));
    }

    #[test]
    fn rejects_wrong_length() {
        assert!("1".parse::<Address32>().is_err());
    }
}
