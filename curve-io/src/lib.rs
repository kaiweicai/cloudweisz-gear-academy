
#![no_std]

use gstd::{prelude::*, ActorId, MessageId};
use gmeta::{In, InOut, Metadata, Out};

pub struct CurveMetadata;

impl Metadata for CurveMetadata{
    type Init = In<InitConfig>;
    type Handle = ();

    type Reply = ();
    type Others = ();
    type Signal = ();
    type State = ();
}

#[derive(Debug,Decode,Encode,TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct InitConfig{
    pub virtual_sui_amt: u128,
    pub target_supply_threshold: u64, // trade amount is engough to on board.
    pub migration_fee: u64,
    pub listing_fee: u128,
    pub swap_fee_ratio: u64,
    pub total_supply_limit: u128, // limit the total supply. only fit this rules can join this curve
    pub admin: ActorId,
    pub fund_manager: ActorId,
}

#[derive(Debug,Decode,Encode,TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum FTAction{
    Listing{
        symbol:String,
        coin_address:ActorId,
        twitter:String,
        telegram:String,
        website:String,
        migrate_price:u128,
    },
    Buy{
        symbol:String,
        expect_token_output_amount:u128,

    }
}