use crate::error::*;
use crate::utils::hex_str_to_bytes;
use crate::{address::Address, utils::ArrayString};
use bech32::Variant;
use bech32::{self, FromBase32, ToBase32};
use ripemd::Ripemd160 as Ripemd;
use sha2::Digest as Sha2Digest;
use sha2::Sha256;
use std::fmt::{self, Display, Formatter};
use std::hash::Hash;
use std::str::FromStr;

/// Represents a public key of a given private key in the Cosmos Network.
#[derive(PartialEq, Eq, Copy, Clone, Hash)]
pub struct PublicKey {
    bytes: [u8; 33],
    prefix: ArrayString,
}

impl PublicKey {
    /// In cases where it's impossible to know the Bech32 prefix
    /// we fall back to this value
    pub const DEFAULT_PREFIX: &'static str = "cosmospub";

    /// Create a public key using a slice of bytes
    pub fn from_slice<T: Into<String>>(bytes: &[u8], prefix: T) -> Result<Self, PublicKeyError> {
        if bytes.len() != 33 {
            return Err(PublicKeyError::BytesDecodeErrorWrongLength);
        }
        let mut result = [0u8; 33];
        result.copy_from_slice(bytes);
        PublicKey::from_bytes(result, prefix)
    }

    /// Create a public key using an array of bytes
    pub fn from_bytes<T: Into<String>>(
        bytes: [u8; 33],
        prefix: T,
    ) -> Result<PublicKey, PublicKeyError> {
        Ok(PublicKey {
            bytes,
            prefix: ArrayString::new(&prefix.into())?,
        })
    }

    /// Returns bytes of a given public key as a slice of bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.bytes.to_vec()
    }

    pub fn get_prefix(&self) -> String {
        self.prefix.to_string()
    }

    pub fn change_prefix<T: Into<String>>(&mut self, prefix: T) -> Result<(), PublicKeyError> {
        self.prefix = ArrayString::new(&prefix.into())?;
        Ok(())
    }

    /// Create an address object using a given public key.
    pub fn to_address(&self) -> Address {
        let current_prefix = self.get_prefix();
        // Cosmos has the format cosmospub -> cosmos which we
        // attempt to keep the convention here, note that other
        // conventions may come out with the wrong prefix by default
        // that's up to the caller to fix
        let new_prefix = if current_prefix.ends_with("pub") {
            current_prefix.trim_end_matches("pub")
        } else {
            &current_prefix
        };
        // unwrap, the only failure possibility is if the Prefix is bad
        // and our own prefix can't possibly be bad, we've already validated it
        // and only reduced it's length since then
        self.to_address_with_prefix(new_prefix).unwrap()
    }

    /// Create an address object using a given public key with the given prefix
    /// provided as a utility for one step creation and change of prefix if the conventions
    /// in `to_address()` are incorrect
    pub fn to_address_with_prefix(&self, prefix: &str) -> Result<Address, AddressError> {
        let sha256 = Sha256::digest(self.bytes);
        let ripemd160 = Ripemd::digest(sha256);
        let mut bytes: [u8; 20] = Default::default();
        bytes.copy_from_slice(&ripemd160[..]);
        Address::from_bytes(bytes, prefix)
    }

    /// Creates amino representation of a given public key.
    ///
    /// It is used internally for bech32 encoding.
    pub fn to_amino_bytes(&self) -> Vec<u8> {
        let mut key_bytes = vec![0xEB, 0x5A, 0xE9, 0x87, 0x21];
        key_bytes.extend(self.as_bytes());
        key_bytes
    }

    /// Create a bech32 encoded public key with an arbitrary prefix
    ///
    /// * `hrp` - A prefix for a bech32 encoding. By a convention
    /// Cosmos Network uses `cosmospub` as a prefix for encoding public keys.
    pub fn to_bech32<T: Into<String>>(&self, hrp: T) -> Result<String, PublicKeyError> {
        let bech32 = bech32::encode(
            &hrp.into(),
            self.to_amino_bytes().to_base32(),
            Variant::Bech32,
        )?;
        Ok(bech32)
    }

    /// Parse a bech32 encoded public key
    ///
    /// * `s` - A bech32 encoded public key
    pub fn from_bech32(s: String) -> Result<PublicKey, PublicKeyError> {
        let (hrp, data, _) = match bech32::decode(&s) {
            Ok(val) => val,
            Err(_e) => return Err(PublicKeyError::Bech32InvalidEncoding),
        };
        let vec: Vec<u8> = match FromBase32::from_base32(&data) {
            Ok(val) => val,
            Err(_e) => return Err(PublicKeyError::Bech32InvalidBase32),
        };
        let mut key = [0u8; 33];
        if vec.len() != 38 {
            return Err(PublicKeyError::Bech32WrongLength);
        }
        // the amnio representation prepends 5 bytes, we truncate those here
        // see to_amino_bytes()
        key.copy_from_slice(&vec[5..]);
        PublicKey::from_bytes(key, hrp)
    }
}

impl FromStr for PublicKey {
    type Err = PublicKeyError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(k) = PublicKey::from_bech32(s.to_string()) {
            Ok(k)
        } else if let Ok(bytes) = hex_str_to_bytes(s) {
            if bytes.len() == 33 {
                let mut inner = [0; 33];
                inner.copy_from_slice(&bytes[0..33]);
                PublicKey::from_bytes(inner, PublicKey::DEFAULT_PREFIX)
            } else {
                Err(PublicKeyError::HexDecodeErrorWrongLength)
            }
        } else {
            match base64::Engine::decode(&base64::engine::general_purpose::STANDARD_NO_PAD, s) {
                Ok(bytes) => {
                    if bytes.len() == 33 {
                        let mut inner = [0; 33];
                        inner.copy_from_slice(&bytes[0..33]);
                        Ok(PublicKey::from_bytes(inner, PublicKey::DEFAULT_PREFIX)?)
                    } else {
                        Err(PublicKeyError::BytesDecodeErrorWrongLength)
                    }
                }
                Err(e) => Err(PublicKeyError::Base64DecodeError(e)),
            }
        }
    }
}

impl Display for PublicKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let display = self.to_bech32(self.get_prefix()).unwrap();
        write!(f, "{}", display).expect("Unable to write");
        Ok(())
    }
}

impl fmt::Debug for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_bech32(self.get_prefix()).unwrap())
    }
}

#[test]
fn check_bech32() {
    let raw_bytes = [
        0x02, 0xA1, 0x63, 0x3C, 0xAF, 0xCC, 0x01, 0xEB, 0xFB, 0x6D, 0x78, 0xE3, 0x9F, 0x68, 0x7A,
        0x1F, 0x09, 0x95, 0xC6, 0x2F, 0xC9, 0x5F, 0x51, 0xEA, 0xD1, 0x0A, 0x02, 0xEE, 0x0B, 0xE5,
        0x51, 0xB5, 0xDC,
    ];
    let public_key = PublicKey::from_slice(&raw_bytes, PublicKey::DEFAULT_PREFIX)
        .expect("Unable to create bytes from slice");
    assert_eq!(&public_key.bytes[..], &raw_bytes[..]);
    let res = public_key.to_string();

    // ground truth
    assert_eq!(
        res,
        "cosmospub1addwnpepq2skx090esq7h7md0r3e76r6ruyet330e904r6k3pgpwuzl92x6actrt4uq"
    );

    // pubkey of secp256k1 private key "mySecret"
    let raw_bytes = [
        2, 150, 81, 169, 170, 196, 194, 43, 39, 179, 1, 154, 238, 109, 247, 70, 38, 110, 26, 231,
        70, 238, 121, 119, 42, 110, 94, 173, 25, 142, 189, 7, 195,
    ];
    let public_key = PublicKey::from_slice(&raw_bytes, PublicKey::DEFAULT_PREFIX)
        .expect("Unable to create bytes from slice");
    let res = public_key
        .to_bech32("cosmospub")
        .expect("Unable to convert to bech32");

    assert_eq!(
        res,
        "cosmospub1addwnpepq2t9r2d2cnpzkfanqxdwum0hgcnxuxh8gmh8jae2de026xvwh5ruxuv5let"
    );

    let check: Result<PublicKey, PublicKeyError> =
        "cosmospub1addwnpepq2t9r2d2cnpzkfanqxdwum0hgcnxuxh8gmh8jae2de026xvwh5ruxuv5let".parse();
    assert_eq!(check.unwrap(), public_key)
}

#[test]
fn parse_base64_pubkey() {
    let key = "AvDDT1xY7hXKTy5ESqckNpBbQIArTkf21CfLFDnmWUY4";
    let _key: PublicKey = key.parse().unwrap();
}

#[test]
fn test_default_prefix() {
    PublicKey::from_bytes([0; 33], PublicKey::DEFAULT_PREFIX).unwrap();
}
