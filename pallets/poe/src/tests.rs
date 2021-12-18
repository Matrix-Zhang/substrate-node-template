use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn normal_operation_should_works() {
	new_test_ext().execute_with(|| {
		let claim = b"12345678".to_vec();
		// test create claim
		assert_ok!(Poe::create_claim(Origin::signed(1), claim.clone()));
		// test transfer claim
		assert_ok!(Poe::transfer_claim(Origin::signed(1), 2, claim.clone()));
		// test revoke claim
		assert_ok!(Poe::revoke_claim(Origin::signed(2), claim));
	});
}

#[test]
fn should_return_exist_error_when_create_claim_and_given_an_exist_claim() {
	new_test_ext().execute_with(|| {
		let claim = b"12345678".to_vec();
		assert_ok!(Poe::create_claim(Origin::signed(1), claim.clone()));
		assert_noop!(Poe::create_claim(Origin::signed(1), claim), <Error<Test>>::ProofAlreadyExist);
	});
}

#[test]
fn should_return_too_long_error_when_create_claim_and_given_an_long_claim() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Poe::create_claim(Origin::signed(1), b"123456789".to_vec()),
			<Error<Test>>::ProofTooLong
		);
	});
}

#[test]
fn should_return_not_owner_error_when_not_owner_call_revoke_claim() {
	new_test_ext().execute_with(|| {
		let claim = b"12345678".to_vec();
		assert_ok!(Poe::create_claim(Origin::signed(1), claim.clone()));
		assert_noop!(Poe::revoke_claim(Origin::signed(2), claim), <Error<Test>>::NotClaimOwner);
	});
}

#[test]
fn should_return_not_exist_error_when_revoke_claim_and_given_an_nonexit_claim() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Poe::revoke_claim(Origin::signed(1), b"12345678".to_vec()),
			<Error<Test>>::ProofNotExist
		);
	});
}

#[test]
fn should_return_not_owner_error_when_not_owner_call_transfer_claim() {
	new_test_ext().execute_with(|| {
		let claim = b"12345678".to_vec();
		assert_ok!(Poe::create_claim(Origin::signed(1), claim.clone()));
		assert_noop!(Poe::transfer_claim(Origin::signed(2), 1, claim), <Error<Test>>::NotClaimOwner);
	});
}

#[test]
fn should_return_not_exist_error_when_transfer_claim_and_given_an_nonexit_claim() {
	new_test_ext().execute_with(|| {
		assert_ok!(Poe::create_claim(Origin::signed(1), b"12345678".to_vec()));
		assert_noop!(Poe::transfer_claim(Origin::signed(1), 2, b"87654321".to_vec()), <Error<Test>>::ProofNotExist);

	});
}
