use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};
use enum_dispatch::enum_dispatch;
use pyth_sdk_solana::Price;
use switchboard_on_demand::CurrentResult;

#[repr(u8)]
#[derive(Copy, Clone, Debug, AnchorDeserialize, AnchorSerialize, PartialEq, Eq)]
pub enum OracleSetup {
    None,
    PythLegacy,
    SwitchboardV2,
    PythPushOracle,
    SwitchboardPull,
    StakedWithPythPush,
}
unsafe impl Zeroable for OracleSetup {}
unsafe impl Pod for OracleSetup {}

impl OracleSetup {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::None),
            1 => Some(Self::PythLegacy),
            2 => Some(Self::SwitchboardV2),
            3 => Some(Self::PythPushOracle),
            4 => Some(Self::SwitchboardPull),
            5 => Some(Self::StakedWithPythPush),
            _ => None,
        }
    }
}

#[enum_dispatch(PriceAdapter)]
#[cfg_attr(feature = "client", derive(Clone))]
pub enum OraclePriceFeedAdapter {
    PythLegacy(PythLegacyPriceFeed),
    SwitchboardV2(SwitchboardV2PriceFeed),
    PythPushOracle(PythPushOraclePriceFeed),
    SwitchboardPull(SwitchboardPullPriceFeed),
}

#[cfg_attr(feature = "client", derive(Clone, Debug))]
pub struct SwitchboardV2PriceFeed {
    _ema_price: Box<Price>,
    _price: Box<Price>,
}

// TODO:
// impl SwitchboardV2PriceFeed {}

#[cfg_attr(feature = "client", derive(Clone, Debug))]
pub struct PythLegacyPriceFeed {
    ema_price: Box<Price>,
    price: Box<Price>,
}

// TODO:
// impl PythLegacyPriceFeed {
//     pub fn load_checked(ai: &AccountInfo, current_time: i64, max_age: u64) -> MarginfiResult<Self> {
//         let price_feed = load_pyth_price_feed(ai)?;

//         let ema_price = if live!() {
//             price_feed
//                 .get_ema_price_no_older_than(current_time, max_age)
//                 .ok_or(MarginfiError::InternalLogicError)?
//         } else {
//             price_feed.get_ema_price_unchecked()
//         }
//     }
// }

#[cfg_attr(feature = "client", derive(Clone, Debug))]
pub struct PythPushOraclePriceFeed {
    ema_price: Box<pyth_solana_receiver_sdk::price_update::Price>,
    price: Box<pyth_solana_receiver_sdk::price_update::Price>,
}

// TODO:
// impl PythPushOraclePriceFeed {}

#[cfg_attr(feature = "client", derive(Clone, Debug))]
pub struct SwitchboardPullPriceFeed {
    pub feed: Box<LitePullFeedAccountData>,
}

// TODO:
// impl SwitchboardPullPriceFeed {}

#[cfg_attr(feature = "client", derive(Clone, Debug))]
pub struct LitePullFeedAccountData {
    pub result: CurrentResult,
    #[cfg(feature = "client")]
    pub feed_hash: [u8; 32],
    #[cfg(feature = "client")]
    pub last_update_timestamp: i64,
}