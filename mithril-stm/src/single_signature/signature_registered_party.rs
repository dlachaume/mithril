use blake2::digest::{Digest, FixedOutput};
use serde::{Deserialize, Serialize, Serializer, ser::SerializeTuple};

use crate::key_registration::RegisteredParty;
use crate::{SingleSignature, StmSignatureError};

/// Signature with its registered party.
#[derive(Debug, Clone, Hash, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
pub struct SingleSignatureWithRegisteredParty {
    /// Stm signature
    pub sig: SingleSignature,
    /// Registered party
    pub reg_party: RegisteredParty,
}

impl SingleSignatureWithRegisteredParty {
    /// Convert `SingleSignatureWithRegisteredParty` to bytes
    /// # Layout
    /// * RegParty
    /// * Signature
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&self.reg_party.to_bytes());
        out.extend_from_slice(&self.sig.to_bytes());

        out
    }
    ///Extract a `SingleSignatureWithRegisteredParty` from a byte slice.
    pub fn from_bytes<D: Digest + Clone + FixedOutput>(
        bytes: &[u8],
    ) -> Result<SingleSignatureWithRegisteredParty, StmSignatureError> {
        let reg_party = RegisteredParty::from_bytes(
            bytes.get(0..104).ok_or(StmSignatureError::SerializationError)?,
        )?;
        let sig = SingleSignature::from_bytes::<D>(
            bytes.get(104..).ok_or(StmSignatureError::SerializationError)?,
        )?;

        Ok(SingleSignatureWithRegisteredParty { sig, reg_party })
    }
}

impl Serialize for SingleSignatureWithRegisteredParty {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut tuple = serializer.serialize_tuple(2)?;
        tuple.serialize_element(&self.sig)?;
        tuple.serialize_element(&self.reg_party)?;
        tuple.end()
    }
}
