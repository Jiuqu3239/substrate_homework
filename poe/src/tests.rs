use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn test_create_claim() {
	new_test_ext().execute_with(|| {
		let claim=vec![0,1,2,3,4];

		assert_ok!(PoeModule::create_claim(RuntimeOrigin::signed(10086), claim));
	});
}

#[test]
fn test_revoke_claim() {
	new_test_ext().execute_with(|| {
		let claim: Vec<u8> = vec![1,2,3,4,5,6];

		assert_ok!(PoeModule::create_claim(RuntimeOrigin::signed(10086), claim.clone()));

		assert_ok!(PoeModule::revoke_claim(RuntimeOrigin::signed(10086), claim.clone()));
	});
}

#[test]
fn test_revoke_claim_failed() {
	new_test_ext().execute_with(|| {
		let claim: Vec<u8> = vec![1,2,1,2,2];
		assert_noop!(
			PoeModule::revoke_claim(RuntimeOrigin::signed(10086), claim),
			Error::<Test>::NotClaimOwner
		);
	});
}

#[test]
fn test_transfer_claim() {
	new_test_ext().execute_with(|| {
		let claim: Vec<u8> = vec![1,3,1,2,3,4];
		assert_ok!(PoeModule::create_claim(RuntimeOrigin::signed(10086), claim.clone()));
		assert_ok!(PoeModule::transfer_claim(RuntimeOrigin::signed(10086), claim.clone(), 10000));
	});
}

#[test]
fn test_transfer_claim_failed() {
	new_test_ext().execute_with(|| {
		let claim: Vec<u8> = vec![1,0,2,4,2,0,4,8];

		assert_noop!(
			PoeModule::transfer_claim(RuntimeOrigin::signed(10086), claim, 10001),
			Error::<Test>::NotClaimOwner
		);
	});
}