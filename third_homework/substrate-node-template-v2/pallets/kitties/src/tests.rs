use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};

#[test]
fn it_works_for_create() {
	new_test_ext().execute_with(|| {
		let kitty_id = 0;
		let account_id = 1;

		#[allow(unused_must_use)]
		{
			Balances::force_set_balance(RuntimeOrigin::root(), account_id, 1_000_000_000);
		}

		assert_eq!(KittiesModule::next_kitty_id(), kitty_id);
		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(account_id), *b"aaaa0000"));

		assert_eq!(KittiesModule::next_kitty_id(), kitty_id + 1);
		assert_eq!(KittiesModule::kitties(kitty_id).is_some(), true);
		assert_eq!(KittiesModule::kitty_owner(kitty_id), Some(account_id));
		assert_eq!(KittiesModule::kitty_parents(kitty_id), None);

		crate::NextKittyId::<Test>::set(crate::KittyId::max_value());

		assert_noop!(
			KittiesModule::create(RuntimeOrigin::signed(account_id), *b"aaaa0000"),
			Error::<Test>::InvalidKittyId
		);
	});
}

#[test]
fn it_works_for_breed() {
	new_test_ext().execute_with(|| {
		let kitty_id = 0;
		let account_id = 1;

		#[allow(unused_must_use)]
		{
			Balances::force_set_balance(RuntimeOrigin::root(), account_id, 1_000_000_000);
		}

		assert_noop!(
			KittiesModule::breed(
				RuntimeOrigin::signed(account_id),
				kitty_id,
				kitty_id,
				*b"aaaa0000"
			),
			Error::<Test>::SameKittyId
		);

		assert_noop!(
			KittiesModule::breed(
				RuntimeOrigin::signed(account_id),
				kitty_id,
				kitty_id + 1,
				*b"aaaa0000"
			),
			Error::<Test>::InvalidKittyId
		);

		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(account_id), *b"aaaa0000"));
		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(account_id), *b"aaaa0000"));

		assert_eq!(KittiesModule::next_kitty_id(), kitty_id + 2);

		assert_ok!(KittiesModule::breed(
			RuntimeOrigin::signed(account_id),
			kitty_id,
			kitty_id + 1,
			*b"aaaa0000"
		));

		let breed_kitty_id = 2;

		assert_eq!(KittiesModule::next_kitty_id(), breed_kitty_id + 1);

		assert_eq!(KittiesModule::kitties(breed_kitty_id).is_some(), true);

		assert_eq!(KittiesModule::kitty_owner(breed_kitty_id), Some(account_id));

		assert_eq!(KittiesModule::kitty_parents(breed_kitty_id), Some((kitty_id, kitty_id + 1)));
	});
}

#[test]
fn it_works_for_transfer() {
	new_test_ext().execute_with(|| {
		let kitty_id = 0;
		let account_id = 1;
		let recipient = 2;

		#[allow(unused_must_use)]
		{
			Balances::force_set_balance(RuntimeOrigin::root(), account_id, 1_000_000_000);
			Balances::force_set_balance(RuntimeOrigin::root(), recipient, 1_000_000_000);
		}

		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(account_id), *b"aaaa0000"));
		assert_eq!(KittiesModule::kitty_owner(kitty_id), Some(account_id));

		assert_noop!(
			KittiesModule::transfer(RuntimeOrigin::signed(recipient), account_id, kitty_id),
			Error::<Test>::NotOwner
		);

		assert_ok!(KittiesModule::transfer(RuntimeOrigin::signed(account_id), recipient, kitty_id));

		assert_eq!(KittiesModule::kitty_owner(kitty_id), Some(recipient));

		assert_ok!(KittiesModule::transfer(RuntimeOrigin::signed(recipient), account_id, kitty_id));

		assert_eq!(KittiesModule::kitty_owner(kitty_id), Some(account_id));
	})
}

#[test]
fn it_works_for_create_event() {
	new_test_ext().execute_with(|| {
		let kitty_id = 0;
		let account_id = 1;

		#[allow(unused_must_use)]
		{
			Balances::force_set_balance(RuntimeOrigin::root(), account_id, 1_000_000_000);
		}

		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(account_id), *b"aaaa0000"));

		System::assert_has_event(
			Event::KittyCreated {
				who: account_id,
				kitty_id,
				kitty: KittiesModule::kitties(kitty_id).unwrap(),
			}
			.into(),
		);
	});
}

#[test]
fn it_works_for_bred_event() {
	new_test_ext().execute_with(|| {
		let kitty_id = 0;
		let account_id = 1;

		#[allow(unused_must_use)]
		{
			Balances::force_set_balance(RuntimeOrigin::root(), account_id, 1_000_000_000);
		}

		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(account_id), *b"aaaa0000"));
		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(account_id), *b"aaaa0000"));
		assert_ok!(KittiesModule::breed(
			RuntimeOrigin::signed(account_id),
			kitty_id,
			kitty_id + 1,
			*b"aaaa0000"
		));

		System::assert_has_event(
			Event::KittyBred {
				who: account_id,
				kitty_id: kitty_id + 2,
				kitty: KittiesModule::kitties(kitty_id + 2).unwrap(),
			}
			.into(),
		);
	});
}

#[test]
fn it_works_for_transferred_event() {
	new_test_ext().execute_with(|| {
		let kitty_id = 0;
		let account_id = 1;
		let recipient = 2;

		#[allow(unused_must_use)]
		{
			Balances::force_set_balance(RuntimeOrigin::root(), account_id, 1_000_000_000);
			Balances::force_set_balance(RuntimeOrigin::root(), recipient, 1_000_000_000);
		}

		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(account_id), *b"aaaa0000"));
		assert_ok!(KittiesModule::transfer(RuntimeOrigin::signed(account_id), recipient, kitty_id));

		System::assert_has_event(
			Event::KittyTransferred { who: account_id, recipient, kitty_id }.into(),
		);
	});
}

#[test]
fn it_works_for_sale() {
	new_test_ext().execute_with(|| {
		let kitty_id = 0;
		let account_id = 1;
		let account_id_2 = 2;
		#[allow(unused_must_use)]
		{
			Balances::force_set_balance(RuntimeOrigin::root(), account_id, 1_000_000_000);
		}

		assert_noop!(
			KittiesModule::sale(RuntimeOrigin::signed(account_id), kitty_id),
			Error::<Test>::InvalidKittyId
		);

		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(account_id), *b"hoodyboo"));

		assert_noop!(
			KittiesModule::sale(RuntimeOrigin::signed(account_id_2), kitty_id),
			Error::<Test>::NotOwner
		);

		assert_ok!(KittiesModule::sale(RuntimeOrigin::signed(account_id), kitty_id));
		assert!(KittiesModule::kitty_on_sale(kitty_id).is_some());

		System::assert_has_event(Event::KittyOnSale { who: account_id, kitty_id }.into());

		assert_noop!(
			KittiesModule::sale(RuntimeOrigin::signed(account_id), kitty_id),
			Error::<Test>::AlreadyOnSale
		);
	});
}

#[test]
fn it_works_for_buy() {
	new_test_ext().execute_with(|| {
		let kitty_id = 0;
		let account_id = 1;
		let buy_account_id = 2;

		#[allow(unused_must_use)]
		{
			Balances::force_set_balance(RuntimeOrigin::root(), account_id, 1_000_000_000);
			Balances::force_set_balance(RuntimeOrigin::root(), buy_account_id, 1_000_000_000);
		}

		assert_noop!(
			KittiesModule::buy(RuntimeOrigin::signed(account_id), kitty_id),
			Error::<Test>::InvalidKittyId
		);

		assert_ok!(KittiesModule::create(RuntimeOrigin::signed(account_id), *b"helobudy"));

		assert_noop!(
			KittiesModule::buy(RuntimeOrigin::signed(buy_account_id), kitty_id),
			Error::<Test>::NotOnSale
		);

		assert_ok!(KittiesModule::sale(RuntimeOrigin::signed(account_id), kitty_id));

		assert_noop!(
			KittiesModule::buy(RuntimeOrigin::signed(account_id), kitty_id),
			Error::<Test>::AlreadyOwned
		);

		assert_ok!(KittiesModule::buy(RuntimeOrigin::signed(buy_account_id), kitty_id));

		System::assert_has_event(Event::KittyBought { who: buy_account_id, kitty_id }.into());

		assert!(KittiesModule::kitty_on_sale(kitty_id).is_none());

		assert_eq!(KittiesModule::kitty_owner(kitty_id), Some(buy_account_id));
	});
}
