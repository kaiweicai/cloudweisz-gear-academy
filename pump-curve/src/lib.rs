
#![no_std]
use curve_io::InitConfig;
use gstd::{ActorId, msg, prelude::*};

pub struct Configurator{
    virtual_sui_amt: u64,
    target_supply_threshold: u64, // trade amount is enough to on board.
    migration_fee: u64,
    listing_fee: u64,
    swap_fee_ratio: u64,
    total_supply_limit: u64, // limit the total supply. only fit this rules can join this curve
    pub admin: ActorId,
    pub fund_manager: ActorId,
}

static mut PUMP_CURVE: Option<Configurator> = None;

#[no_mangle]
extern fn init() {
    let init_config:InitConfig = msg::load().expect("Failed to load payload");
    unsafe {
        PUMP_CURVE = Some(Configurator{
            virtual_sui_amt: init_config.virtual_sui_amt,
            target_supply_threshold: init_config.target_supply_threshold,
            migration_fee: init_config.migration_fee,
            listing_fee: init_config.listing_fee,
            swap_fee_ratio: init_config.swap_fee_ratio,
            total_supply_limit: init_config.total_supply_limit,
            admin: init_config.admin,
            fund_manager: init_config.admin,
        });
    }
}

#[no_mangle]
extern fn handle(){

}
