use super::*;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok, BoundedVec};


const ACCOUNT_ID_1: u64 = 1;
const ACCOUNT_ID_2: u64 = 2;
const ACCOUNT_ID_3: u64 = 3;

fn new_claim() -> BoundedVec<u8, ConstU32<10>> {
	return BoundedVec::try_from(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]).unwrap()
}



#[test]
fn create_claim_works() {

new_test_ext().execute_with(|| {
   // let claim = BoundedVec::try_from(vec![0, 1]).unwrap();
   let claim = vec![0, 1];

   let bound_claim = BoundedVec::try_from(vec![0,1]).unwrap();

    assert_ok!(PoeModule::create_claim(RuntimeOrigin::signed(1), bound_claim.clone()));
    
  
    assert_eq!(
        Proofs::<Test>::get(&bound_claim),
        Some((1,frame_system::Pallet::<Test>::block_number())),
    )
})
}

#[test]
fn create_claim_failed_when_claim_already_exist() {
    new_test_ext().execute_with(|| {
       // let bound_claim = BoundedVec::try_from(vec![0, 1]).unwrap();
        let bound_claim: BoundedVec<u8, ConstU32<10>> = BoundedVec::try_from(vec![0,1]).unwrap();

        let _ = PoeModule::create_claim(RuntimeOrigin::signed(1), bound_claim.clone());

        assert_noop!(
            PoeModule::create_claim(RuntimeOrigin::signed(1), bound_claim.clone()),
            Error::<Test>::ProofAlreadyExist
        );

    })
}

#[test]
fn revoke_claim_works() {
    new_test_ext().execute_with(|| {
        let bound_claim: BoundedVec<u8, ConstU32<10>> = BoundedVec::try_from(vec![0,1]).unwrap();

        let _ = PoeModule::create_claim(RuntimeOrigin::signed(1), bound_claim.clone());

        assert_ok!(PoeModule::revoke_claim(RuntimeOrigin::signed(1), bound_claim.clone()));
    })
    
}

#[test]
fn revoke_claim_faild_when_claim_not_exist() {
    new_test_ext().execute_with(|| {
        let bound_claim: BoundedVec<u8, ConstU32<10>> = BoundedVec::try_from(vec![0,1]).unwrap();
        assert_noop!(
            PoeModule::revoke_claim(RuntimeOrigin::signed(1), bound_claim.clone()),
            Error::<Test>::ClaimNotExist
        );
    })
}

#[test]
fn revoke_claim_faild_with_wrong_owner() {
    new_test_ext().execute_with(|| {
        let bound_claim: BoundedVec<u8, ConstU32<10>> = BoundedVec::try_from(vec![0,1]).unwrap();
        let _ = PoeModule::create_claim(RuntimeOrigin::signed(1), claim.clone());

        assert_noop!(
            PoeModule::revoke_claim(RuntimeOrigin::signed(2), bound_claim.clone()),
            Error::<Test>::NotClaimOwner
        );

    })
}

#[test]
fn transfer_claim_works() {
	new_test_ext().execute_with(|| {
		let claim = new_claim();
		let signer = RuntimeOrigin::signed(ACCOUNT_ID_1);

		// 创建存证
		assert_ok!(PoeModule::create_claim(signer.clone(), claim.clone()));
		// 转移存证
		assert_ok!(PoeModule::transfer_claim(signer, ACCOUNT_ID_2, claim.clone()));
		// 检查存证
		//assert_eq!(PoeModule::proofs(&claim), Some((ACCOUNT_ID_2, System::block_number())));
	})
}

#[test]
fn transfer_claim_failed_when_claim_not_exist() {
	new_test_ext().execute_with(|| {
		let claim = new_claim();
		let signer = RuntimeOrigin::signed(ACCOUNT_ID_1);

		// 转移存证
		assert_noop!(
			PoeModule::transfer_claim(signer, ACCOUNT_ID_2, claim.clone()),
			Error::<Test>::ClaimNotExist
		);
		// 检查存证
		assert_eq!(PoeModule::proofs(&claim), None);
	})
}

#[test]
fn transfer_claim_failed_when_not_owner() {
	new_test_ext().execute_with(|| {
		let claim = new_claim();
		let signer = RuntimeOrigin::signed(ACCOUNT_ID_1);
		let signer_2 = RuntimeOrigin::signed(ACCOUNT_ID_2);

		// 创建存证
		assert_ok!(PoeModule::create_claim(signer, claim.clone()));
		// 转移存证
		assert_noop!(
			PoeModule::transfer_claim(signer_2, ACCOUNT_ID_3, claim.clone()),
			Error::<Test>::NotClaimOwner
		);
		// 检查存证
		assert_eq!(PoeModule::proofs(&claim), Some((ACCOUNT_ID_1, System::block_number())));
	})
}

#[test]
fn transfer_claim_failed_when_transfer_to_owner() {
	new_test_ext().execute_with(|| {
		let claim = new_claim();
		let signer = RuntimeOrigin::signed(ACCOUNT_ID_1);

		// 创建存证
		assert_ok!(PoeModule::create_claim(signer.clone(), claim.clone()));
		// 转移存证
		assert_noop!(
			PoeModule::transfer_claim(signer, ACCOUNT_ID_1, claim.clone()),
			Error::<Test>::TransferToOwner
		);
		// 检查存证
		assert_eq!(PoeModule::proofs(&claim), Some((ACCOUNT_ID_1, System::block_number())));
	})
}
