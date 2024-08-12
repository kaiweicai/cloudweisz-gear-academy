#![no_std]

use gmeta::{In, InOut, Metadata, Out};
use gstd::{ActorId, prelude::*};


pub struct CoinMetadata;

impl Metadata for CoinMetadata {
    type Init = In<InitConfig>;
    type Handle = InOut<FTAction, FTEvent>;
    type Reply = ();
    type Others = ();
    type Signal = ();
    type State = Out<IoToken>;
}

#[derive(Debug,Decode,Encode,TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct InitConfig{
    pub name:String,
    pub symbol:String,
    pub decimals:u8
}

#[derive(Debug,Decode,Encode,TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum FTAction{
    Mint(u128),
    Burn(u128),
    Transfer{from:ActorId,to:ActorId,amount:u128},
    Approve{to:ActorId,amount:u128},
    TotalSupply,    //? is not a coin-state query?
    BalanceOf(ActorId), //? is not a coin-state query?
}

#[derive(Debug, Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum FTEvent {
    Transfer {
        from: ActorId,
        to: ActorId,
        amount: u128,
    },
    Approve {
        from: ActorId,
        to: ActorId,
        amount: u128,
    },
    TotalSupply(u128),
    Balance(u128),
}

#[derive(Debug,Decode,Encode,TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct  IoToken{
    pub name:String,
    pub symbol:String,
    pub decimals:u8,
    pub total_supply:u128,
    pub balances:Vec<(ActorId,u128)>,
    pub allowances:Vec<(ActorId,Vec<(ActorId,u128)>)>,
}