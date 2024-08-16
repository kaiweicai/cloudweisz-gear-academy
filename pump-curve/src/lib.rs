#![no_std]

use curve_io::{FTAction, InitConfig};
use gstd::{collections::HashMap, msg, prelude::*, ActorId};

pub struct Configurator {
    virtual_sui_amt: u128,
    target_supply_threshold: u64, // trade amount is enough to on board.
    migration_fee: u64,
    listing_fee: u128,
    swap_fee_ratio: u64,
    total_supply_limit: u128, // limit the total supply. only fit this rules can join this curve
    pub admin: ActorId,
    pub fund_manager: ActorId,
    pop_coin: HashMap<String, BondingCurve>,
}

impl Configurator {
    pub fn get_token_output_amount(&self, input_amount: u128, symbol: &str) -> u128 {
        let coin_amount = self.pop_coin.get(symbol).unwrap().coin_amount;
        let native_coin_amount = self.get_virtual_base_coin(symbol);
        let coin_reverse = coin_amount;
        input_amount * coin_reverse / (native_coin_amount + input_amount)
    }

    pub fn get_virtual_base_coin(&self, symbol: &str) -> u128 {
        let native_amount = self.pop_coin.get(symbol).unwrap().native_amount;
        native_amount + self.virtual_sui_amt
    }

    pub fn get_reserves(&self,symbol: &str)->(u128,u128){
        let pop_coin = self.pop_coin.get(symbol).expect("coin not exist");
        (pop_coin.coin_amount,pop_coin.native_amount)
    }
}

pub struct BondingCurve {
    token_balance: u128,
    is_active: bool,
    creator: ActorId,
    twitter: String,
    telegram: String,
    website: String,
    // target_supply_threshold:u64, //depracate
    migration_target: u128,
    coin_amount: u128,
    native_amount: u128,
}

static mut PUMP_CURVE: Option<Configurator> = None;

#[no_mangle]
extern fn init() {
    let init_config: InitConfig = msg::load().expect("Failed to load payload");
    unsafe {
        PUMP_CURVE = Some(Configurator {
            virtual_sui_amt: init_config.virtual_sui_amt,
            target_supply_threshold: init_config.target_supply_threshold,
            migration_fee: init_config.migration_fee,
            listing_fee: init_config.listing_fee,
            swap_fee_ratio: init_config.swap_fee_ratio,
            total_supply_limit: init_config.total_supply_limit,
            admin: init_config.admin,
            fund_manager: init_config.admin,
            pop_coin: Default::default(),
        });
    }
}

#[no_mangle]
extern fn handle() {
    let action: FTAction = msg::load().expect("Failed to load payload");
    let pump_curve = unsafe { PUMP_CURVE.as_mut().expect("The program is not initialized") };
    match action {
        FTAction::Listing {
            symbol,
            coin_address,
            twitter,
            telegram,
            website,
            migrate_price,
        } => {
            // 检查coin是否已经上架
            let pop_coin = &pump_curve.pop_coin;
            assert!(pop_coin.get(&symbol).is_none(), "coin is exist!");

            // 保证token的量全部存入了cure中，没有剩余的量
            // 存入token到fundManager中
            let fund_manager = pump_curve.fund_manager;
            let total_supply_limit = pump_curve.total_supply_limit;

            msg::send(
                coin_address,
                coin_io::FTAction::Transfer {
                    from: msg::source(),
                    to: fund_manager,
                    amount: total_supply_limit,
                },
                0,
            )
            .expect("Failed to send");
            //收取一定的手续费到当前合约。
            let value = msg::value();
            assert!(value >= pump_curve.listing_fee, "listing fee not equal");
            // 生成上线记录
            let bound_curve = BondingCurve {
                token_balance: total_supply_limit,
                is_active: true,
                creator: msg::source(),
                twitter: twitter.clone(),
                telegram,
                website,
                migration_target: migrate_price,
                coin_amount: 0,
                native_amount: 0,
            };
            pump_curve.pop_coin.insert(twitter, bound_curve);
        }
        FTAction::Buy {
            symbol,
            coin_address,
            expect_token_output_amount,
        } => {
            let bounding_curve = pump_curve
                .pop_coin
                .get(&symbol)
                .expect("symbol is not exist");

            // take fee
            let input_amount = msg::value();
            let token_output_amount = pump_curve.get_token_output_amount(input_amount, &symbol);
            assert!(
                token_output_amount >= expect_token_output_amount,
                "expect less than min output "
            );
            pump_curve.pop_coin.entry(symbol).and_modify(|curve| {
                curve.native_amount += input_amount;
                curve.coin_amount -= token_output_amount;
            });
            // 转账给用户。此处可能转账失败。因为没有授权
            let fund_manager = pump_curve.fund_manager;
            msg::send(
                coin_address,
                coin_io::FTAction::Transfer {
                    from: fund_manager,
                    to: msg::source(),
                    amount: token_output_amount,
                },
                0,
            );
            //检查是否被买空

        }
    }
}
