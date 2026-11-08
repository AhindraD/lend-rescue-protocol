use borsh::{BorshDeserialize, BorshSerialize};

use super::error::PythError;

pub const PYTH_RECEIVER_PROGRAM_ID: &str = "rec5EKMGg6MxZYaMdyBfgwp4d5rB9T1VQH5pJv5LtFJ";
pub const PRICE_UPDATE_V2_DISCRIMINATOR: [u8; 8] = [34, 241, 35, 99, 157, 126, 244, 205];
pub const LEGACY_PYTH_MAGIC: u32 = 0xa1b2c3d4;

/// Verification security level in Pyth PriceUpdateV2.
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Eq)]
pub enum VerificationLevel {
    Partial { num_signatures: u8 },
    Full,
}

/// Inner price feed message payload.
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct PriceFeedMessage {
    pub feed_id: [u8; 32],
    pub price: i64,
    pub conf: u64,
    pub expo: i32,
    pub publish_time: i64,
    pub prev_publish_time: i64,
    pub ema_price: i64,
    pub ema_conf: u64,
}

/// Official Pyth Core persistent on-chain price update account layout (Anchor PriceUpdateV2).
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct PriceUpdateV2 {
    pub write_authority: [u8; 32],
    pub verification_level: VerificationLevel,
    pub price_message: PriceFeedMessage,
    pub posted_slot: u64,
}

impl PriceUpdateV2 {
    /// Deserializes raw account data slice verifying the 8-byte Anchor discriminator.
    pub fn try_from_slice(data: &[u8]) -> Result<Self, PythError> {
        if data.len() < 8 {
            return Err(PythError::InvalidAccountData(
                "Account data too short for PriceUpdateV2 discriminator".to_string(),
            ));
        }
        if data[..8] != PRICE_UPDATE_V2_DISCRIMINATOR {
            return Err(PythError::InvalidAccountData(format!(
                "Invalid PriceUpdateV2 discriminator: expected {:?}, got {:?}",
                PRICE_UPDATE_V2_DISCRIMINATOR,
                &data[..8]
            )));
        }
        let mut slice = &data[8..];
        BorshDeserialize::deserialize(&mut slice).map_err(|e| {
            PythError::InvalidAccountData(format!(
                "PriceUpdateV2 Borsh deserialization failed: {}",
                e
            ))
        })
    }

    /// Serializes PriceUpdateV2 with leading Anchor discriminator.
    pub fn to_account_bytes(&self) -> Result<Vec<u8>, PythError> {
        let mut bytes = Vec::with_capacity(146);
        bytes.extend_from_slice(&PRICE_UPDATE_V2_DISCRIMINATOR);
        self.serialize(&mut bytes).map_err(|e| {
            PythError::InvalidAccountData(format!(
                "PriceUpdateV2 Borsh serialization failed: {}",
                e
            ))
        })?;
        Ok(bytes)
    }
}

/// Unified decoded Pyth observation payload across PriceUpdateV2 and Legacy accounts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedPythData {
    pub feed_id: Option<[u8; 32]>,
    pub price: i64,
    pub conf: u64,
    pub expo: i32,
    pub publish_time: i64,
    pub slot: u64,
    pub is_legacy: bool,
}

/// Decodes Pyth account bytes, supporting both PriceUpdateV2 and Legacy Pyth formats.
pub fn try_decode_pyth_account(data: &[u8]) -> Result<DecodedPythData, PythError> {
    if data.len() < 8 {
        return Err(PythError::InvalidAccountData(
            "Account data too short to determine Pyth format".to_string(),
        ));
    }

    // Check for PriceUpdateV2 discriminator
    if data[..8] == PRICE_UPDATE_V2_DISCRIMINATOR {
        let account = PriceUpdateV2::try_from_slice(data)?;
        return Ok(DecodedPythData {
            feed_id: Some(account.price_message.feed_id),
            price: account.price_message.price,
            conf: account.price_message.conf,
            expo: account.price_message.expo,
            publish_time: account.price_message.publish_time,
            slot: account.posted_slot,
            is_legacy: false,
        });
    }

    // Check for Legacy Pyth magic (0xa1b2c3d4)
    if data.len() >= 248 {
        let magic = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        if magic == LEGACY_PYTH_MAGIC {
            let expo = i32::from_le_bytes([data[20], data[21], data[22], data[23]]);
            let price = i64::from_le_bytes([
                data[208], data[209], data[210], data[211], data[212], data[213], data[214],
                data[215],
            ]);
            let conf = u64::from_le_bytes([
                data[216], data[217], data[218], data[219], data[220], data[221], data[222],
                data[223],
            ]);
            let pub_slot = u64::from_le_bytes([
                data[232], data[233], data[234], data[235], data[236], data[237], data[238],
                data[239],
            ]);
            let timestamp = i64::from_le_bytes([
                data[240], data[241], data[242], data[243], data[244], data[245], data[246],
                data[247],
            ]);

            return Ok(DecodedPythData {
                feed_id: None,
                price,
                conf,
                expo,
                publish_time: timestamp,
                slot: pub_slot,
                is_legacy: true,
            });
        }
    }

    Err(PythError::InvalidAccountData(
        "Unrecognized Pyth account format or invalid discriminator".to_string(),
    ))
}
