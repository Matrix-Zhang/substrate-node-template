use frame_support::{assert_noop, assert_ok};
use sp_core::H256;

use crate::{mock::*, Error};

#[test]
fn create_kitty_should_work() {
	new_test_ext().execute_with(|| {
		setup_blocks(10);
		assert_ok!(Kitties::create_kitty(Origin::signed(1)));
		assert_eq!(Balances::reserved_balance(1), 5);
		assert_eq!(Balances::free_balance(1), 15);
	});
}

#[test]
fn should_return_exceed_max_kitty_owned_error_when_create_kitty() {
	new_test_ext().execute_with(|| {
		setup_blocks(10);
		for _ in 0..MaxKittyOwned::get() {
			assert_ok!(Kitties::create_kitty(Origin::signed(1)));
		}
		assert_noop!(Kitties::create_kitty(Origin::signed(1)), <Error<Test>>::ExceedMaxKittyOwned);
	});
}

#[test]
fn should_return_not_enough_balance_error_when_create_kitty() {
	new_test_ext().execute_with(|| {
		setup_blocks(10);
		assert_noop!(Kitties::create_kitty(Origin::signed(3)), <Error<Test>>::NotEnoughBalance);
	});
}

#[test]
fn set_price_should_work() {
	new_test_ext().execute_with(|| {
		setup_blocks(10);
		assert_ok!(Kitties::create_kitty(Origin::signed(1)));
		let kitties = Kitties::kittes_owned(&1);
		assert_eq!(kitties.len(), 1);
		assert_ok!(Kitties::set_price(Origin::signed(1), kitties[0], Some(1)));
	});
}

#[test]
fn should_return_kitty_not_exist_error_when_set_price() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Kitties::set_price(Origin::signed(1), H256::zero(), Some(1)),
			<Error<Test>>::KittyNotExist
		);
	});
}

#[test]
fn transfer_kitty_should_work() {
	new_test_ext().execute_with(|| {
		setup_blocks(10);
		assert_ok!(Kitties::create_kitty(Origin::signed(1)));
		let kitties = Kitties::kittes_owned(&1);
		assert_eq!(kitties.len(), 1);
		assert_ok!(Kitties::transfer(Origin::signed(1), 2, kitties[0]));
	});
}

#[test]
fn should_return_kitty_not_exist_error_when_transfer_kitty() {
	new_test_ext().execute_with(|| {
		setup_blocks(10);
		assert_ok!(Kitties::create_kitty(Origin::signed(1)));
		assert_noop!(
			Kitties::transfer(Origin::signed(1), 2, H256::zero()),
			<Error<Test>>::KittyNotExist
		);
	});
}

#[test]
fn should_return_not_kitty_owner_error_when_transfer_kitty() {
	new_test_ext().execute_with(|| {
		setup_blocks(10);
		assert_ok!(Kitties::create_kitty(Origin::signed(1)));
		let kitties = Kitties::kittes_owned(&1);
		assert_eq!(kitties.len(), 1);
		assert_noop!(
			Kitties::transfer(Origin::signed(2), 1, kitties[0]),
			<Error<Test>>::NotKittyOwner
		);
	});
}

#[test]
fn should_return_transfer_self_error_when_transfer_kitty() {
	new_test_ext().execute_with(|| {
		setup_blocks(10);
		assert_ok!(Kitties::create_kitty(Origin::signed(1)));
		let kitties = Kitties::kittes_owned(&1);
		assert_eq!(kitties.len(), 1);
		assert_noop!(
			Kitties::transfer(Origin::signed(1), 1, kitties[0]),
			<Error<Test>>::TransferToSelf
		);
	});
}

#[test]
fn buy_kitty_should_work() {
	new_test_ext().execute_with(|| {
		setup_blocks(10);
		assert_ok!(Kitties::create_kitty(Origin::signed(1)));
		let kitties = Kitties::kittes_owned(&1);
		assert_eq!(kitties.len(), 1);
		assert_ok!(Kitties::set_price(Origin::signed(1), kitties[0], Some(1)));
		assert_ok!(Kitties::buy_kitty(Origin::signed(2), kitties[0], 2));
	});
}

#[test]
fn should_return_kitty_not_exist_error_when_buy_kitty() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Kitties::buy_kitty(Origin::signed(1), H256::zero(), 2),
			<Error<Test>>::KittyNotExist
		);
	});
}

#[test]
fn should_return_kitty_not_for_sale_error_when_buy_kitty() {
	new_test_ext().execute_with(|| {
		setup_blocks(10);
		assert_ok!(Kitties::create_kitty(Origin::signed(1)));
		let kitties = Kitties::kittes_owned(&1);
		assert_eq!(kitties.len(), 1);
		assert_noop!(
			Kitties::buy_kitty(Origin::signed(2), kitties[0], 2),
			<Error<Test>>::KittyNotForSale
		);
	});
}

#[test]
fn should_return_kitty_bid_price_too_low_error_when_buy_kitty() {
	new_test_ext().execute_with(|| {
		setup_blocks(10);
		assert_ok!(Kitties::create_kitty(Origin::signed(1)));
		let kitties = Kitties::kittes_owned(&1);
		assert_eq!(kitties.len(), 1);
		assert_ok!(Kitties::set_price(Origin::signed(1), kitties[0], Some(3)));
		assert_noop!(
			Kitties::buy_kitty(Origin::signed(2), kitties[0], 2),
			<Error<Test>>::KittyBidPriceTooLow
		);
	});
}

#[test]
fn should_return_not_enough_balance_error_when_buy_kitty() {
	new_test_ext().execute_with(|| {
		setup_blocks(10);
		assert_ok!(Kitties::create_kitty(Origin::signed(1)));
		let kitties = Kitties::kittes_owned(&1);
		assert_eq!(kitties.len(), 1);
		assert_ok!(Kitties::set_price(Origin::signed(1), kitties[0], Some(1)));
		assert_noop!(
			Kitties::buy_kitty(Origin::signed(3), kitties[0], 3),
			<Error<Test>>::NotEnoughBalance
		);
	});
}

#[test]
fn breed_kitty_should_work() {
	new_test_ext().execute_with(|| {
		setup_blocks(10);
		assert_ok!(Kitties::create_kitty(Origin::signed(1)));
		assert_ok!(Kitties::create_kitty(Origin::signed(1)));
		let kitties = Kitties::kittes_owned(&1);
		assert_eq!(kitties.len(), 2);
		assert_ok!(Kitties::breed_kitty(Origin::signed(1), kitties[0], kitties[1]));
		let kitties = Kitties::kittes_owned(&1);
		assert_eq!(kitties.len(), 3);
	});
}

#[test]
fn should_return_not_kitty_owner_error_when_breed_kitty() {
	new_test_ext().execute_with(|| {
		setup_blocks(10);
		assert_ok!(Kitties::create_kitty(Origin::signed(1)));
		assert_ok!(Kitties::create_kitty(Origin::signed(1)));
		let kitties = Kitties::kittes_owned(&1);
		assert_eq!(kitties.len(), 2);
		assert_noop!(
			Kitties::breed_kitty(Origin::signed(2), kitties[0], kitties[1]),
			<Error<Test>>::NotKittyOwner
		);
	});
}

#[test]
fn should_return_kitty_not_exist_error_when_breed_kitty() {
	new_test_ext().execute_with(|| {
		setup_blocks(10);
		assert_ok!(Kitties::create_kitty(Origin::signed(1)));
		assert_ok!(Kitties::create_kitty(Origin::signed(1)));
		let kitties = Kitties::kittes_owned(&1);
		assert_eq!(kitties.len(), 2);
		assert_noop!(
			Kitties::breed_kitty(Origin::signed(1), kitties[0], H256::zero()),
			<Error<Test>>::KittyNotExist
		);
	});
}
