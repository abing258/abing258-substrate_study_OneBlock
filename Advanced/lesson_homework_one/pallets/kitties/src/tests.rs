use std::ops::Add;

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::{new_test_ext, KittiesModule, Origin, System, Test};

#[test]
fn it_works_for_creating_kitty() {
	new_test_ext().execute_with(|| {
		// 当块高等于0的时候，不允许生成随机数
		System::set_block_number(1);
		let account_id: u64 = 0;
		let kitty_id: u32 = NextKittyId::<Test>::get();
		assert_ok!(KittiesModule::create(Origin::signed(account_id)));
		assert_eq!(KittyOwner::<Test>::get(kitty_id), Some(account_id));
		assert_ne!(Kitties::<Test>::get(kitty_id), None);
		assert_eq!(NextKittyId::<Test>::get(), kitty_id.add(&1));
		assert_eq!(
			<Test as Config>::Currency::reserved_balance(&account_id),
			<Test as Config>::KittyPrice::get()
		);
	});
}

#[test]
fn create_kitty_fails_for_kitty_id_overflow() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let account_id: u64 = 0;
		let max_index = <Test as Config>::KittyIndex::max_value();
		NextKittyId::<Test>::set(max_index);
		assert_noop!(
			KittiesModule::create(Origin::signed(account_id)),
			Error::<Test>::KittyIdOverflow
		);
	});
}

#[test]
fn create_kitty_fails_for_not_enough_balance() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let account_id: u64 = 2;
		assert_noop!(
			KittiesModule::create(Origin::signed(account_id)),
			Error::<Test>::NotEnoughBalance
		);
	});
}

#[test]
fn create_kitty_fails_for_too_many_kitties() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let account_id: u64 = 0;
		assert_ok!(KittiesModule::create(Origin::signed(account_id)));
		assert_ok!(KittiesModule::create(Origin::signed(account_id)));
		assert_ok!(KittiesModule::create(Origin::signed(account_id)));

		assert_noop!(KittiesModule::create(Origin::signed(account_id)), Error::<Test>::OwnTooManyKitties);
	});
}

#[test]
fn it_works_for_breeding_kitty() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let mut owned_kitty_amount = 0;
		let account_id: u64 = 0;

		let kitty_id_1 = NextKittyId::<Test>::get();
		assert_ok!(KittiesModule::create(Origin::signed(account_id)));
		owned_kitty_amount += 1;

		let kitty_id_2 = NextKittyId::<Test>::get();
		assert_ok!(KittiesModule::create(Origin::signed(account_id)));
		owned_kitty_amount += 1;

		let new_breed_kitty_id = NextKittyId::<Test>::get();
		assert_ok!(KittiesModule::breed(Origin::signed(account_id), kitty_id_1, kitty_id_2));
		owned_kitty_amount += 1;

		assert_eq!(KittyOwner::<Test>::get(new_breed_kitty_id), Some(account_id));
		assert_ne!(Kitties::<Test>::get(new_breed_kitty_id), None);
		assert_eq!(NextKittyId::<Test>::get(), new_breed_kitty_id.add(&1));
		assert_eq!(
			<Test as Config>::Currency::reserved_balance(&account_id),
			<Test as Config>::KittyPrice::get().checked_mul(owned_kitty_amount).unwrap()
		);

	});
}

#[test]
fn breed_kitty_fails_for_not_enough_balance() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let account_id: u64 = 1;
		let kitty_id_1 = NextKittyId::<Test>::get();
		assert_ok!(KittiesModule::create(Origin::signed(account_id)));

		let kitty_id_2 = NextKittyId::<Test>::get();
		assert_ok!(KittiesModule::create(Origin::signed(account_id)));

		assert_noop!(
			KittiesModule::breed(Origin::signed(account_id), kitty_id_1, kitty_id_2),
			Error::<Test>::NotEnoughBalance
		);
	});
}

#[test]
fn breed_kitty_fails_for_same_kitty_id() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let account_id: u64 = 1;
		let kitty_id_1 = NextKittyId::<Test>::get();
		assert_ok!(KittiesModule::create(Origin::signed(account_id)));

		assert_noop!(
			KittiesModule::breed(Origin::signed(account_id), kitty_id_1, kitty_id_1),
			Error::<Test>::SameKittyId
		);
	});
}

#[test]
fn breed_kitty_fails_for_not_exist_kitty_id() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let account_id: u64 = 0;

		let kitty_id_1 = NextKittyId::<Test>::get();
		assert_ok!(KittiesModule::create(Origin::signed(account_id)));

		assert_ok!(KittiesModule::create(Origin::signed(account_id)));

		// INFO: next kitty id after creation 2 kitties - not exist
		let not_exist_kitty_id_2 = NextKittyId::<Test>::get();

		assert_noop!(
			KittiesModule::breed(Origin::signed(account_id), kitty_id_1, not_exist_kitty_id_2),
			Error::<Test>::NotExistKittyId
		);
	});
}

#[test]
fn breed_kitty_fails_for_too_many_kitties() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let account_id: u64 = 0;

		let kitty_id_1 = NextKittyId::<Test>::get();
		assert_ok!(KittiesModule::create(Origin::signed(account_id)));

		let kitty_id_2 = NextKittyId::<Test>::get();
		assert_ok!(KittiesModule::create(Origin::signed(account_id)));

		// Generate 3rd kitty - maximum amount
		assert_ok!(KittiesModule::create(Origin::signed(account_id)));

		assert_noop!(
			KittiesModule::breed(Origin::signed(account_id), kitty_id_1, kitty_id_2),
			Error::<Test>::OwnTooManyKitties
		);
	});
}

#[test]
fn it_works_for_transferring_kitty() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let account_id_1: u64 = 0;
		let account_id_2: u64 = 1;

		let kitty_id = NextKittyId::<Test>::get();
		assert_ok!(KittiesModule::create(Origin::signed(account_id_1)));

		assert_ok!(KittiesModule::transfer(Origin::signed(account_id_1), kitty_id, account_id_2));

		assert_eq!(KittyOwner::<Test>::get(kitty_id), Some(account_id_2));
		assert_ne!(Kitties::<Test>::get(kitty_id), None);
		assert_eq!(NextKittyId::<Test>::get(), kitty_id.add(&1));
		assert_eq!(<Test as Config>::Currency::reserved_balance(&account_id_1), 0);
		assert_eq!(
			<Test as Config>::Currency::reserved_balance(&account_id_2),
			<Test as Config>::KittyPrice::get()
		);

		// TODO: figure out why assert_has_event keep failling with 0 event.
		// System::assert_has_event(TestEvent::KittiesModule(Event::KittyTransferred(
		// 	account_id_1,
		// 	account_id_2,
		// 	kitty_id,
		// )));
	});
}

#[test]
fn transfer_kitty_fails_for_not_enough_balance() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let account_id_1: u64 = 0;
		let account_id_2: u64 = 2;
		let kitty_id_1 = NextKittyId::<Test>::get();
		assert_ok!(KittiesModule::create(Origin::signed(account_id_1)));

		assert_noop!(
			KittiesModule::transfer(Origin::signed(account_id_1), kitty_id_1, account_id_2),
			Error::<Test>::NotEnoughBalance
		);
	});
}

#[test]
fn transfer_kitty_fails_for_not_owner() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let account_id_1: u64 = 0;
		let account_id_2: u64 = 1;
		let kitty_id_1 = NextKittyId::<Test>::get();
		assert_ok!(KittiesModule::create(Origin::signed(account_id_2)));

		assert_noop!(
			KittiesModule::transfer(Origin::signed(account_id_1), kitty_id_1, account_id_2),
			Error::<Test>::NotOwner
		);
	});
}

#[test]
fn transfer_kitty_fails_for_too_many_kitties() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let account_id_1: u64 = 1;
		let account_id_2: u64 = 0;
		let kitty_id_1 = NextKittyId::<Test>::get();
		assert_ok!(KittiesModule::create(Origin::signed(account_id_1)));
		assert_ok!(KittiesModule::create(Origin::signed(account_id_2)));
		assert_ok!(KittiesModule::create(Origin::signed(account_id_2)));
		assert_ok!(KittiesModule::create(Origin::signed(account_id_2)));

		assert_noop!(
			KittiesModule::transfer(Origin::signed(account_id_1), kitty_id_1, account_id_2),
			Error::<Test>::OwnTooManyKitties
		);
	});
}