#![no_std]

use curve_io::{CurveEvent, FTAction, InitConfig};
use gstd::{collections::HashMap, msg, prelude::*, ActorId};

pub struct Configurator {
    virtual_sui_amt: u128,
    target_supply_threshold: u128, // trade amount is enough to on board.
    migration_fee: u128,
    listing_fee: u128,
    swap_fee_ratio: u128,
    total_supply_limit: u128, // limit the total supply. only fit this rules can join this curve
    pub admin: ActorId,
    pub fund_manager: ActorId,
    pop_coin: HashMap<String, BondingCurve>,
}


impl BondingCurve {
    pub fn get_token_output_amount(&self, input_amount: u128,virtual_sui_amt:u128) -> u128 {
        let coin_amount = self.coin_amount;
        let native_coin_amount = self.get_virtual_base_coin(virtual_sui_amt);
        let coin_reverse = coin_amount;
        input_amount * coin_reverse / (native_coin_amount + input_amount)
    }

    pub fn get_virtual_base_coin(&self,virtual_sui_amt:u128) -> u128 {
        let native_amount = self.native_amount;
        native_amount + virtual_sui_amt
    }

    pub fn get_reserves(&self)->(u128,u128){
        (self.coin_amount,self.native_amount)
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
                .get_mut(&symbol)
                .expect("symbol is not exist");
            // 检查curve是否有效
            assert!(bounding_curve.is_active, "curve is not active");
            // take fee
            let input_amount = msg::value();
            let virtual_sui_amt = pump_curve.virtual_sui_amt;
            let token_output_amount = bounding_curve.get_token_output_amount(input_amount,virtual_sui_amt);
            assert!(
                token_output_amount >= expect_token_output_amount,
                "expect less than min output "
            );
            bounding_curve.native_amount += input_amount;
            bounding_curve.coin_amount -= token_output_amount;
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
            let (coin_amount, native_amount) = bounding_curve.get_reserves();
            assert!(coin_amount>=0&&native_amount>=0,"coin or native amount not enough");
            // 判断是否达到上架标准
            if coin_amount <= pump_curve.target_supply_threshold {
                let bounding_curve = pump_curve
                    .pop_coin
                    .get_mut(&symbol)
                    .expect("symbol is not exist");
                bounding_curve.is_active = false;
                // TODO 可以上架进行交易
                msg::reply(CurveEvent::MigrationPendingEvent {
                    symbol,
                    sui_reserve_val: native_amount,
                    token_reserve_val: coin_amount,
                },0);

            }
            msg::reply(curve_io::CurveEvent::SwapEvent {
                is_buy: true,
                input_amount,
                output_amount: token_output_amount,
                native_reserve_val: native_amount,
                token_reserve_val: coin_amount,
                sender: msg::source(),
            },0);
        }
    }
}
