// Copyright 2019 Conflux Foundation. All rights reserved.
// Conflux is free software and distributed under GNU General Public License.
// See http://www.gnu.org/licenses/

use super::OverlayAccount;
use cfx_parameters::staking::*;
use cfx_statedb::StateDb;
use cfx_storage::{
    tests::new_state_manager_for_unit_test, StorageManagerTrait,
};
use cfx_types::{address_util::AddressUtil, Address, AddressSpaceUtil, U256};
use keccak_hash::KECCAK_EMPTY;
use primitives::{
    account::ContractAccount, storage::STORAGE_LAYOUT_REGULAR_V0, Account,
    SponsorInfo, VoteStakeList,
};

use crate::state::get_state_for_genesis_write;
use primitives::is_default::IsDefault;
use std::str::FromStr;

fn test_account_is_default(account: &mut OverlayAccount) {
    let state = get_state_for_genesis_write();

    assert!(account.as_account().is_default());

    account.cache_ext_fields(true, true, &state.db).unwrap();
    assert!(account.vote_stake_list().is_default());
    assert!(account.deposit_list().is_default());
}

#[test]
fn new_overlay_account_is_default() {
    let normal_addr =
        Address::from_str("1000000000000000000000000000000000000000")
            .unwrap()
            .with_native_space();
    let builtin_addr =
        Address::from_str("0000000000000000000000000000000000000000")
            .unwrap()
            .with_native_space();

    test_account_is_default(&mut OverlayAccount::new_basic(
        &normal_addr,
        U256::zero(),
    ));
    test_account_is_default(&mut OverlayAccount::new_basic(
        &builtin_addr,
        U256::zero(),
    ));
}

#[test]
fn test_overlay_account_create() {
    let mut address = Address::random();
    address.set_user_account_type_bits();
    let address_with_space = address.with_native_space();
    let account = Account::new_empty_with_balance(
        &address_with_space,
        &U256::zero(),
        &U256::zero(),
    );
    // test new from account 1
    let overlay_account =
        OverlayAccount::from_loaded(&address_with_space, account);
    assert!(overlay_account.deposit_list.is_none());
    assert!(overlay_account.vote_stake_list.is_none());
    assert_eq!(overlay_account.address().address, address);
    assert_eq!(*overlay_account.balance(), 0.into());
    assert_eq!(*overlay_account.nonce(), 0.into());
    assert_eq!(*overlay_account.staking_balance(), 0.into());
    assert_eq!(overlay_account.collateral_for_storage(), 0.into());
    assert_eq!(*overlay_account.accumulated_interest_return(), 0.into());
    assert_eq!(overlay_account.code_hash(), KECCAK_EMPTY);
    assert_eq!(overlay_account.is_newly_created_contract(), false);
    assert_eq!(*overlay_account.admin(), Address::zero());
    assert_eq!(*overlay_account.sponsor_info(), Default::default());

    let mut contract_addr = Address::random();
    contract_addr.set_contract_type_bits();
    let contract_addr_with_space = contract_addr.with_native_space();
    let mut user_addr = Address::random();
    user_addr.set_user_account_type_bits();
    let user_addr_with_space = user_addr.with_native_space();
    let admin = Address::random();
    let sponsor_info = SponsorInfo {
        sponsor_for_gas: Address::random(),
        sponsor_for_collateral: Address::random(),
        sponsor_balance_for_gas: U256::from(123),
        sponsor_balance_for_collateral: U256::from(124),
        sponsor_gas_bound: U256::from(2),
        storage_points: None,
    };
    let account = Account::from_contract_account(
        contract_addr,
        ContractAccount {
            balance: 101.into(),
            nonce: 55.into(),
            code_hash: KECCAK_EMPTY,
            staking_balance: 11111.into(),
            collateral_for_storage: 455.into(),
            accumulated_interest_return: 2.into(),
            admin,
            sponsor_info: sponsor_info.clone(),
        },
    );

    // test new from account 2
    let overlay_account =
        OverlayAccount::from_loaded(&contract_addr_with_space, account);
    assert!(overlay_account.deposit_list.is_none());
    assert!(overlay_account.vote_stake_list.is_none());
    assert_eq!(overlay_account.address().address, contract_addr);
    assert_eq!(*overlay_account.balance(), 101.into());
    assert_eq!(*overlay_account.nonce(), 55.into());
    assert_eq!(*overlay_account.staking_balance(), 11111.into());
    assert_eq!(overlay_account.collateral_for_storage(), 455.into());
    assert_eq!(*overlay_account.accumulated_interest_return(), 2.into());
    assert_eq!(overlay_account.code_hash(), KECCAK_EMPTY);
    assert_eq!(overlay_account.is_newly_created_contract(), false);
    assert_eq!(*overlay_account.admin(), admin);
    assert_eq!(*overlay_account.sponsor_info(), sponsor_info);

    // test new basic
    let overlay_account =
        OverlayAccount::new_basic(&user_addr_with_space, 1011.into());
    assert!(overlay_account.deposit_list.is_none());
    assert!(overlay_account.vote_stake_list.is_none());
    assert_eq!(overlay_account.address().address, user_addr);
    assert_eq!(*overlay_account.balance(), 1011.into());
    assert_eq!(*overlay_account.staking_balance(), 0.into());
    assert_eq!(overlay_account.collateral_for_storage(), 0.into());
    assert_eq!(*overlay_account.accumulated_interest_return(), 0.into());
    assert_eq!(overlay_account.code_hash(), KECCAK_EMPTY);
    assert_eq!(overlay_account.is_newly_created_contract(), false);
    assert_eq!(overlay_account.is_contract(), false);
    assert_eq!(overlay_account.is_basic(), true);
    assert_eq!(*overlay_account.admin(), Address::zero());
    assert_eq!(*overlay_account.sponsor_info(), Default::default());

    // test new contract
    let mut overlay_account = OverlayAccount::new_contract(
        &contract_addr.with_native_space(),
        5678.into(),
        false,
        Some(STORAGE_LAYOUT_REGULAR_V0),
    );
    assert!(overlay_account.deposit_list.is_none());
    assert!(overlay_account.vote_stake_list.is_none());
    assert_eq!(overlay_account.address().address, contract_addr);
    assert_eq!(*overlay_account.balance(), 5678.into());
    assert_eq!(*overlay_account.staking_balance(), 0.into());
    assert_eq!(overlay_account.collateral_for_storage(), 0.into());
    assert_eq!(*overlay_account.accumulated_interest_return(), 0.into());
    assert_eq!(overlay_account.code_hash(), KECCAK_EMPTY);
    assert_eq!(overlay_account.is_newly_created_contract(), true);
    assert_eq!(overlay_account.is_contract(), true);
    assert_eq!(
        overlay_account.storage_layout_change(),
        Some(&STORAGE_LAYOUT_REGULAR_V0)
    );
    assert_eq!(*overlay_account.admin(), Address::zero());
    assert_eq!(*overlay_account.sponsor_info(), Default::default());
    overlay_account.inc_nonce();

    // test new contract with admin
    let overlay_account = OverlayAccount::new_contract_with_admin(
        &contract_addr_with_space,
        5678.into(),
        &admin,
        false,
        Some(STORAGE_LAYOUT_REGULAR_V0),
        false,
    );
    assert!(overlay_account.deposit_list.is_none());
    assert!(overlay_account.vote_stake_list.is_none());
    assert_eq!(overlay_account.address().address, contract_addr);
    assert_eq!(*overlay_account.balance(), 5678.into());
    assert_eq!(*overlay_account.staking_balance(), 0.into());
    assert_eq!(overlay_account.collateral_for_storage(), 0.into());
    assert_eq!(*overlay_account.accumulated_interest_return(), 0.into());
    assert_eq!(overlay_account.code_hash(), KECCAK_EMPTY);
    assert_eq!(overlay_account.is_newly_created_contract(), true);
    assert_eq!(overlay_account.is_contract(), true);
    assert_eq!(
        overlay_account.storage_layout_change(),
        Some(&STORAGE_LAYOUT_REGULAR_V0)
    );
    assert_eq!(*overlay_account.admin(), admin);
    assert_eq!(*overlay_account.sponsor_info(), Default::default());
}

#[test]
fn test_deposit_and_withdraw() {
    let storage_manager = new_state_manager_for_unit_test();
    let db = StateDb::new(storage_manager.get_state_for_genesis_write());
    let mut address = Address::random();
    address.set_user_account_type_bits();
    let address_with_space = address.with_native_space();
    let account = Account::new_empty_with_balance(
        &address_with_space,
        &U256::zero(),
        &U256::zero(),
    );
    let mut accumulated_interest_rate = vec![*ACCUMULATED_INTEREST_RATE_SCALE];
    for _ in 0..100000 {
        let last = *accumulated_interest_rate.last().unwrap();
        accumulated_interest_rate.push(
            last * (*INITIAL_INTEREST_RATE_PER_BLOCK
                + *INTEREST_RATE_PER_BLOCK_SCALE)
                / *INTEREST_RATE_PER_BLOCK_SCALE,
        );
    }
    let mut overlay_account =
        OverlayAccount::from_loaded(&address_with_space, account);
    overlay_account
        .cache_ext_fields(
            true, /* cache_deposit_list */
            true, /* cache_vote_list */
            &db,
        )
        .unwrap();
    assert!(overlay_account.deposit_list.is_some());
    assert!(overlay_account.vote_stake_list.is_some());
    // add balance 2 * 10^15
    overlay_account.add_balance(&2_000_000_000_000_000u64.into());
    assert_eq!(
        *overlay_account.balance(),
        U256::from(2_000_000_000_000_000u64)
    );
    assert_eq!(*overlay_account.staking_balance(), U256::zero());
    assert_eq!(
        overlay_account.withdrawable_staking_balance(0 /* timestamp */),
        U256::zero()
    );
    // deposit
    overlay_account.deposit(
        1_000_000_000_000_000u64.into(), /* amount */
        accumulated_interest_rate[1],
        1,     /* deposit_time */
        false, /* cip97 */
    );
    assert_eq!(
        *overlay_account.balance(),
        U256::from(1_000_000_000_000_000u64)
    );
    assert_eq!(
        *overlay_account.staking_balance(),
        U256::from(1_000_000_000_000_000u64)
    );
    assert_eq!(
        overlay_account.withdrawable_staking_balance(0 /* timestamp */),
        U256::from(1_000_000_000_000_000u64)
    );
    overlay_account.deposit(
        100_000_000_000_000u64.into(), /* amount */
        accumulated_interest_rate[2],
        2,     /* deposit_time */
        false, /* cip97 */
    );
    assert_eq!(
        *overlay_account.balance(),
        U256::from(900_000_000_000_000u64)
    );
    assert_eq!(
        *overlay_account.staking_balance(),
        U256::from(1_100_000_000_000_000u64)
    );
    assert_eq!(
        overlay_account.withdrawable_staking_balance(3 /* timestamp */),
        U256::from(1_100_000_000_000_000u64)
    );
    overlay_account.deposit(
        10_000_000_000_000u64.into(), /* amount */
        accumulated_interest_rate[3],
        3,     /* deposit_time */
        false, /* cip97 */
    );
    assert_eq!(
        *overlay_account.balance(),
        U256::from(890_000_000_000_000u64)
    );
    assert_eq!(
        *overlay_account.staking_balance(),
        U256::from(1_110_000_000_000_000u64)
    );
    assert_eq!(
        overlay_account.withdrawable_staking_balance(4 /* timestamp */),
        U256::from(1_110_000_000_000_000u64)
    );
    overlay_account.deposit(
        1_000_000_000_000u64.into(), /* amount */
        accumulated_interest_rate[4],
        4,     /* deposit_time */
        false, /* cip97 */
    );
    assert_eq!(
        *overlay_account.balance(),
        U256::from(889_000_000_000_000u64)
    );
    assert_eq!(
        *overlay_account.staking_balance(),
        U256::from(1_111_000_000_000_000u64)
    );
    assert_eq!(
        overlay_account.withdrawable_staking_balance(5 /* timestamp */),
        U256::from(1_111_000_000_000_000u64)
    );
    overlay_account.deposit(
        100_000_000_000u64.into(), /* amount */
        accumulated_interest_rate[5],
        5,     /* deposit_time */
        false, /* cip97 */
    );
    assert_eq!(
        *overlay_account.balance(),
        U256::from(888_900_000_000_000u64)
    );
    assert_eq!(
        *overlay_account.staking_balance(),
        U256::from(1_111_100_000_000_000u64)
    );
    assert_eq!(
        overlay_account.withdrawable_staking_balance(6 /* timestamp */),
        U256::from(1_111_100_000_000_000u64)
    );
    overlay_account.deposit(
        10_000_000_000u64.into(), /* amount */
        accumulated_interest_rate[6],
        6,     /* deposit_time */
        false, /* cip97 */
    );
    assert_eq!(
        *overlay_account.balance(),
        U256::from(888_890_000_000_000u64)
    );
    assert_eq!(
        *overlay_account.staking_balance(),
        U256::from(1_111_110_000_000_000u64)
    );
    assert_eq!(
        overlay_account.withdrawable_staking_balance(7 /* timestamp */),
        U256::from(1_111_110_000_000_000u64)
    );
    overlay_account.deposit(
        1_000_000_000u64.into(), /* amount */
        accumulated_interest_rate[7],
        7,     /* deposit_time */
        false, /* cip97 */
    );
    assert_eq!(
        *overlay_account.balance(),
        U256::from(888_889_000_000_000u64)
    );
    assert_eq!(
        *overlay_account.staking_balance(),
        U256::from(1_111_111_000_000_000u64)
    );
    assert_eq!(
        overlay_account.withdrawable_staking_balance(8 /* timestamp */),
        U256::from(1_111_111_000_000_000u64)
    );
    assert_eq!(overlay_account.deposit_list().len(), 7);

    // add storage
    assert_eq!(overlay_account.collateral_for_storage(), U256::from(0));
    overlay_account.add_collateral_for_storage(&11116.into());
    assert_eq!(overlay_account.collateral_for_storage(), U256::from(11_116));
    assert_eq!(
        *overlay_account.balance(),
        U256::from(888_888_999_988_884u64)
    );
    assert_eq!(
        *overlay_account.staking_balance(),
        U256::from(1_111_111_000_000_000u64)
    );
    assert_eq!(
        overlay_account.withdrawable_staking_balance(8 /* timestamp */),
        U256::from(1_111_111_000_000_000u64)
    );

    // sub storage
    overlay_account.sub_collateral_for_storage(&11116.into());
    assert_eq!(overlay_account.collateral_for_storage(), U256::zero());
    assert_eq!(
        *overlay_account.balance(),
        U256::from(888_889_000_000_000u64)
    );
    assert_eq!(
        *overlay_account.staking_balance(),
        U256::from(1_111_111_000_000_000u64)
    );
    assert_eq!(
        overlay_account.withdrawable_staking_balance(8 /* timestamp */),
        U256::from(1_111_111_000_000_000u64)
    );

    // withdraw
    // 500_000_000_000_000 from `block_number = 1`
    let interest = overlay_account.withdraw(
        500_000_000_000_000u64.into(), /* amount */
        accumulated_interest_rate[1],
        false,
    );
    assert_eq!(interest, U256::zero());
    assert_eq!(*overlay_account.accumulated_interest_return(), U256::zero());
    assert_eq!(
        *overlay_account.balance(),
        U256::from(1_388_889_000_000_000u64)
    );
    assert_eq!(
        *overlay_account.staking_balance(),
        U256::from(611_111_000_000_000u64)
    );
    assert_eq!(
        overlay_account.withdrawable_staking_balance(8 /* timestamp */),
        U256::from(611_111_000_000_000u64)
    );
    assert_eq!(overlay_account.deposit_list().len(), 7);
    assert_eq!(
        overlay_account.deposit_list()[0].amount,
        U256::from(500_000_000_000_000u64)
    );

    // 500_000_000_000_000 from `block_number = 1`
    let interest = overlay_account.withdraw(
        500_000_000_000_000u64.into(), /* amount */
        accumulated_interest_rate[100000],
        false,
    );
    assert_eq!(interest, U256::from(31_710_480_387u64));
    assert_eq!(
        *overlay_account.accumulated_interest_return(),
        U256::from(31_710_480_387u64)
    );
    assert_eq!(
        *overlay_account.balance(),
        U256::from(1_888_920_710_480_387u64)
    );
    assert_eq!(
        *overlay_account.staking_balance(),
        U256::from(111_111_000_000_000u64)
    );
    assert_eq!(
        overlay_account.withdrawable_staking_balance(8 /* timestamp */),
        U256::from(111_111_000_000_000u64)
    );
    assert_eq!(overlay_account.deposit_list().len(), 6);
    assert_eq!(
        overlay_account.deposit_list()[0].amount,
        U256::from(100_000_000_000_000u64)
    );

    // 100_000_000_000_000 from `block_number = 2`
    // 10_000_000_000_000 from `block_number = 3`
    // 250_000_000_000 from `block_number = 4`
    let interest = overlay_account.withdraw(
        110_250_000_000_000u64.into(), /* amount */
        accumulated_interest_rate[100],
        false,
    );
    assert_eq!(interest, U256::from(6_845_508u64));
    assert_eq!(
        *overlay_account.accumulated_interest_return(),
        U256::from(31_717_325_895u64)
    );
    assert_eq!(
        *overlay_account.balance(),
        U256::from(1_999_170_717_325_895u64)
    );
    assert_eq!(
        *overlay_account.staking_balance(),
        U256::from(861_000_000_000u64)
    );
    assert_eq!(
        overlay_account.withdrawable_staking_balance(8 /* timestamp */),
        U256::from(861_000_000_000u64)
    );
    assert_eq!(overlay_account.deposit_list().len(), 4);
    assert_eq!(
        overlay_account.deposit_list()[0].amount,
        U256::from(750_000_000_000u64)
    );
}

fn check_ordered_feature(vote_stake_list: &VoteStakeList) {
    for i in 1..vote_stake_list.len() {
        assert!(
            vote_stake_list[i - 1].unlock_block_number
                < vote_stake_list[i].unlock_block_number
        );
        assert!(vote_stake_list[i - 1].amount > vote_stake_list[i].amount);
    }
}

fn init_test_account() -> OverlayAccount {
    let storage_manager = new_state_manager_for_unit_test();
    let db = StateDb::new(storage_manager.get_state_for_genesis_write());
    let mut address = Address::random();
    address.set_user_account_type_bits();
    let address_with_space = address.with_native_space();
    let account = Account::new_empty_with_balance(
        &address_with_space,
        &10_000_000.into(),
        &U256::zero(),
    );

    let mut overlay_account =
        OverlayAccount::from_loaded(&address_with_space, account.clone());
    overlay_account
        .cache_ext_fields(
            true, /* cache_deposit_list */
            true, /* cache_vote_list */
            &db,
        )
        .unwrap();
    assert!(overlay_account.deposit_list.is_some());
    assert!(overlay_account.vote_stake_list.is_some());
    overlay_account.deposit(
        10000000.into(), /* amount */
        0.into(),        /* accumulated_interest_rate */
        0,               /* deposit_time */
        false,           /* cip97 */
    );
    overlay_account.vote_lock(
        100000.into(), /* amount */
        10,            /* unlock_block_number */
    );
    overlay_account.vote_lock(
        10000.into(), /* amount */
        30,           /* unlock_block_number */
    );
    overlay_account.vote_lock(
        1000.into(), /* amount */
        100,         /* unlock_block_number */
    );
    overlay_account.vote_lock(
        100.into(), /* amount */
        500,        /* unlock_block_number */
    );
    check_ordered_feature(overlay_account.vote_stake_list());
    overlay_account
}

#[test]
fn test_vote_lock() {
    let mut overlay_account = init_test_account();
    assert_eq!(
        overlay_account.withdrawable_staking_balance(0 /* block_number */),
        U256::from(9900000)
    );
    overlay_account.remove_expired_vote_stake_info(0 /* block_number */);
    assert_eq!(overlay_account.vote_stake_list().len(), 4);
    assert_eq!(
        overlay_account.withdrawable_staking_balance(10 /* block_number */),
        U256::from(9990000)
    );
    overlay_account.remove_expired_vote_stake_info(10 /* block_number */);
    assert_eq!(overlay_account.vote_stake_list().len(), 3);
    let mut overlay_account = init_test_account();
    assert_eq!(
        overlay_account.withdrawable_staking_balance(11 /* block_number */),
        U256::from(9990000)
    );
    overlay_account.remove_expired_vote_stake_info(11 /* block_number */);
    assert_eq!(overlay_account.vote_stake_list().len(), 3);
    let mut overlay_account = init_test_account();
    assert_eq!(
        overlay_account.withdrawable_staking_balance(30 /* block_number */),
        U256::from(9999000)
    );
    overlay_account.remove_expired_vote_stake_info(30 /* block_number */);
    assert_eq!(overlay_account.vote_stake_list().len(), 2);
    let mut overlay_account = init_test_account();
    assert_eq!(
        overlay_account.withdrawable_staking_balance(499 /* block_number */),
        U256::from(9999900)
    );
    overlay_account.remove_expired_vote_stake_info(499 /* block_number */);
    assert_eq!(overlay_account.vote_stake_list().len(), 1);
    let mut overlay_account = init_test_account();
    assert_eq!(
        overlay_account.withdrawable_staking_balance(500 /* block_number */),
        U256::from(10000000)
    );
    overlay_account.remove_expired_vote_stake_info(500 /* block_number */);
    assert_eq!(overlay_account.vote_stake_list().len(), 0);

    let mut overlay_account = init_test_account();
    overlay_account
        .vote_lock(U256::from(1000), 20 /* unlock_block_number */);
    check_ordered_feature(overlay_account.vote_stake_list());
    assert_eq!(
        overlay_account.withdrawable_staking_balance(0 /* block_number */),
        U256::from(9900000)
    );
    overlay_account.remove_expired_vote_stake_info(0 /* block_number */);
    assert_eq!(overlay_account.vote_stake_list().len(), 4);
    overlay_account
        .vote_lock(U256::from(100000), 10 /* unlock_block_number */);
    check_ordered_feature(overlay_account.vote_stake_list());
    assert_eq!(
        overlay_account.withdrawable_staking_balance(0 /* block_number */),
        U256::from(9900000)
    );
    overlay_account.remove_expired_vote_stake_info(0 /* block_number */);
    assert_eq!(overlay_account.vote_stake_list().len(), 4);
    overlay_account
        .vote_lock(U256::from(1000000), 11 /* unlock_block_number */);
    check_ordered_feature(overlay_account.vote_stake_list());
    assert_eq!(
        overlay_account.withdrawable_staking_balance(0 /* block_number */),
        U256::from(9000000)
    );
    overlay_account.remove_expired_vote_stake_info(0 /* block_number */);
    assert_eq!(overlay_account.vote_stake_list().len(), 4);
    assert_eq!(
        overlay_account.vote_stake_list()[0].unlock_block_number,
        U256::from(11)
    );
    overlay_account
        .vote_lock(U256::from(1000000), 13 /* unlock_block_number */);
    check_ordered_feature(overlay_account.vote_stake_list());
    assert_eq!(
        overlay_account.withdrawable_staking_balance(0 /* block_number */),
        U256::from(9000000)
    );
    overlay_account.remove_expired_vote_stake_info(0 /* block_number */);
    assert_eq!(overlay_account.vote_stake_list().len(), 4);
    assert_eq!(
        overlay_account.vote_stake_list()[0].unlock_block_number,
        U256::from(13)
    );
    overlay_account
        .vote_lock(U256::from(2000000), 40 /* unlock_block_number */);
    check_ordered_feature(overlay_account.vote_stake_list());
    assert_eq!(
        overlay_account.withdrawable_staking_balance(0 /* block_number */),
        U256::from(8000000)
    );
    overlay_account.remove_expired_vote_stake_info(0 /* block_number */);
    assert_eq!(overlay_account.vote_stake_list().len(), 3);
    assert_eq!(
        overlay_account.vote_stake_list()[0].unlock_block_number,
        U256::from(40)
    );
    overlay_account
        .vote_lock(U256::from(10), 600 /* unlock_block_number */);
    check_ordered_feature(overlay_account.vote_stake_list());
    assert_eq!(
        overlay_account.withdrawable_staking_balance(0 /* block_number */),
        U256::from(8000000)
    );
    overlay_account.remove_expired_vote_stake_info(0 /* block_number */);
    assert_eq!(overlay_account.vote_stake_list().len(), 4);
    assert_eq!(
        overlay_account.vote_stake_list()[3].unlock_block_number,
        U256::from(600)
    );
    overlay_account
        .vote_lock(U256::from(1000), 502 /* unlock_block_number */);
    check_ordered_feature(overlay_account.vote_stake_list());
    assert_eq!(
        overlay_account.withdrawable_staking_balance(0 /* block_number */),
        U256::from(8000000)
    );
    overlay_account.remove_expired_vote_stake_info(0 /* block_number */);
    assert_eq!(overlay_account.vote_stake_list().len(), 3);
    assert_eq!(
        overlay_account.vote_stake_list()[0].unlock_block_number,
        U256::from(40)
    );
    assert_eq!(
        overlay_account.vote_stake_list()[1].unlock_block_number,
        U256::from(502)
    );
    overlay_account
        .vote_lock(U256::from(3000000), 550 /* unlock_block_number */);
    check_ordered_feature(overlay_account.vote_stake_list());
    assert_eq!(
        overlay_account.withdrawable_staking_balance(0 /* block_number */),
        U256::from(7000000)
    );
    overlay_account.remove_expired_vote_stake_info(0 /* block_number */);
    assert_eq!(overlay_account.vote_stake_list().len(), 2);
    assert_eq!(
        overlay_account.vote_stake_list()[0].unlock_block_number,
        U256::from(550)
    );
    assert_eq!(
        overlay_account.vote_stake_list()[1].unlock_block_number,
        U256::from(600)
    );
}

#[test]
fn test_clone_overwrite() {
    let mut address = Address::random();
    address.set_contract_type_bits();
    let address_with_space = address.with_native_space();
    let admin = Address::random();
    let sponsor_info = SponsorInfo {
        sponsor_for_gas: Address::random(),
        sponsor_for_collateral: Address::random(),
        sponsor_balance_for_gas: U256::from(123),
        sponsor_balance_for_collateral: U256::from(124),
        sponsor_gas_bound: U256::from(2),
        storage_points: None,
    };
    let account1 = Account::from_contract_account(
        address,
        ContractAccount {
            balance: 1000.into(),
            nonce: 123.into(),
            code_hash: KECCAK_EMPTY,
            staking_balance: 10000000.into(),
            collateral_for_storage: 23.into(),
            accumulated_interest_return: 456.into(),
            admin,
            sponsor_info,
        },
    );

    let admin = Address::random();
    let sponsor_info = SponsorInfo {
        sponsor_for_gas: Address::random(),
        sponsor_for_collateral: Address::random(),
        sponsor_balance_for_gas: U256::from(1233),
        sponsor_balance_for_collateral: U256::from(1244),
        sponsor_gas_bound: U256::from(23),
        storage_points: None,
    };
    let account2 = Account::from_contract_account(
        address,
        ContractAccount {
            balance: 1001.into(),
            nonce: 124.into(),
            code_hash: KECCAK_EMPTY,
            staking_balance: 10000001.into(),
            collateral_for_storage: 24.into(),
            accumulated_interest_return: 457.into(),
            admin,
            sponsor_info,
        },
    );

    let mut overlay_account1 =
        OverlayAccount::from_loaded(&address_with_space, account1.clone());
    let mut overlay_account2 =
        OverlayAccount::from_loaded(&address_with_space, account2.clone());
    assert_eq!(account1, overlay_account1.as_account());
    assert_eq!(account2, overlay_account2.as_account());

    overlay_account1.set_storage_simple(vec![0; 32], U256::zero());
    assert_eq!(account1, overlay_account1.as_account());
    assert_eq!(overlay_account1.storage_write_cache.read().len(), 1);
    let overlay_account = overlay_account1.clone_account_for_checkpoint(0);
    assert_eq!(account1, overlay_account.as_account());
    assert_eq!(overlay_account.storage_write_cache.read().len(), 1);

    overlay_account2.set_storage_simple(vec![0; 32], U256::zero());
    overlay_account2.set_storage_simple(vec![1; 32], U256::zero());
    overlay_account1 = overlay_account2;
    assert_ne!(account1, overlay_account1.as_account());
    assert_eq!(account2, overlay_account1.as_account());
    assert_eq!(overlay_account1.storage_write_cache.read().len(), 2);
}
