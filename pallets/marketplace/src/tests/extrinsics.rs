use super::mock::*;
use crate::tests::mock;
use crate::{
    Error, MarketplaceInformation, MarketplaceType, NFTCurrency, NFTCurrencyCombined,
    NFTCurrencyId, SaleInformation, URI,
};
use frame_support::error::BadOrigin;
use frame_support::instances::Instance1;
use frame_support::{assert_noop, assert_ok};
use frame_system::RawOrigin;
use pallet_balances::Error as BalanceError;

const CAPS_ID: NFTCurrencyId = NFTCurrencyId::Caps;
const TIIME_ID: NFTCurrencyId = NFTCurrencyId::Tiime;

type MPT = MarketplaceType;

#[test]
fn list_happy() {
    ExtBuilder::default()
        .caps(vec![(ALICE, 1000), (BOB, 1000)])
        .build()
        .execute_with(|| {
            let alice: mock::Origin = RawOrigin::Signed(ALICE).into();
            let bob: mock::Origin = RawOrigin::Signed(BOB).into();

            // Happy path Public marketplace
            let price = NFTCurrency::Caps(50);
            let series_id = vec![50];
            let nft_id = help::create_nft(alice.clone(), vec![50], Some(series_id.clone()));
            let sale_info = SaleInformation::new(ALICE, price.clone(), 0);

            help::finish_series(alice.clone(), series_id);
            assert_ok!(Marketplace::list(alice.clone(), nft_id, price, Some(0)));
            assert_eq!(Marketplace::nft_for_sale(nft_id), Some(sale_info));
            assert_eq!(NFTs::data(nft_id).unwrap().locked, true);

            // Happy path Private marketplace
            let series_id = vec![51];
            let mkp_id = help::create_mkp(bob.clone(), MPT::Private, 0, vec![1], vec![ALICE]);
            let sale_info = SaleInformation::new(ALICE, price.clone(), mkp_id);
            let nft_id = help::create_nft(alice.clone(), vec![50], Some(series_id.clone()));

            help::finish_series(alice.clone(), series_id);
            let ok = Marketplace::list(alice.clone(), nft_id, price, Some(mkp_id));
            assert_ok!(ok);
            assert_eq!(Marketplace::nft_for_sale(nft_id), Some(sale_info));
            assert_eq!(NFTs::data(nft_id).unwrap().locked, true);
        })
}

#[test]
fn list_unhappy() {
    ExtBuilder::default()
        .caps(vec![(ALICE, 1000), (BOB, 1000)])
        .build()
        .execute_with(|| {
            let alice: mock::Origin = RawOrigin::Signed(ALICE).into();
            let bob: mock::Origin = RawOrigin::Signed(BOB).into();
            let price = NFTCurrency::Caps(50);

            // Unhappy not the NFT owner
            let ok = Marketplace::list(alice.clone(), 10001, price, Some(0));
            assert_noop!(ok, Error::<Test>::NotNftOwner);

            // Unhappy series not completed
            let series_id = vec![50];
            let nft_id = help::create_nft(alice.clone(), vec![50], Some(series_id.clone()));
            let ok = Marketplace::list(alice.clone(), nft_id, price, Some(0));
            assert_noop!(ok, Error::<Test>::SeriesNotCompleted);

            // Unhappy unknown marketplace
            help::finish_series(alice.clone(), series_id);
            let ok = Marketplace::list(alice.clone(), nft_id, price, Some(10001));
            assert_noop!(ok, Error::<Test>::UnknownMarketplace);

            // Unhappy not on the list
            let mkp_id = help::create_mkp(bob.clone(), MPT::Private, 0, vec![1], vec![]);
            let ok = Marketplace::list(alice.clone(), nft_id, price, Some(mkp_id));
            assert_noop!(ok, Error::<Test>::NotAllowed);

            // Unhappy already locked
            help::lock(nft_id);
            let ok = Marketplace::list(alice.clone(), nft_id, price, None);
            assert_noop!(ok, ternoa_nfts::Error::<Test>::Locked);
        })
}

#[test]
fn unlist_happy() {
    ExtBuilder::default()
        .caps(vec![(ALICE, 1000)])
        .build()
        .execute_with(|| {
            let alice: mock::Origin = RawOrigin::Signed(ALICE).into();

            let price = NFTCurrency::Caps(50);
            let series_id = vec![50];
            let nft_id = help::create_nft(alice.clone(), vec![50], Some(series_id.clone()));

            // Happy path
            help::finish_series(alice.clone(), series_id);
            assert_ok!(Marketplace::list(alice.clone(), nft_id, price, Some(0)));
            assert_ok!(Marketplace::unlist(alice.clone(), nft_id));
            assert_eq!(Marketplace::nft_for_sale(nft_id), None);
            assert_eq!(NFTs::data(nft_id).unwrap().locked, false);
        })
}

#[test]
fn unlist_unhappy() {
    ExtBuilder::default()
        .caps(vec![(ALICE, 1000)])
        .build()
        .execute_with(|| {
            let alice: mock::Origin = RawOrigin::Signed(ALICE).into();

            // Unhappy not the NFT owner
            let ok = Marketplace::unlist(alice.clone(), 10001);
            assert_noop!(ok, Error::<Test>::NotNftOwner);

            // Unhappy not listed NFT
            let nft_id = help::create_nft(alice.clone(), vec![50], None);
            let ok = Marketplace::unlist(alice.clone(), nft_id);
            assert_noop!(ok, Error::<Test>::NftNotForSale);
        })
}

#[test]
fn buy_happy() {
    ExtBuilder::default()
        .caps(vec![(ALICE, 1000), (BOB, 1000), (DAVE, 1000)])
        .tiime(vec![(ALICE, 1000), (BOB, 1000), (DAVE, 1000)])
        .build()
        .execute_with(|| {
            let alice: mock::Origin = RawOrigin::Signed(ALICE).into();
            let bob: mock::Origin = RawOrigin::Signed(BOB).into();
            let dave: mock::Origin = RawOrigin::Signed(DAVE).into();

            let nft_id_1 = help::create_nft_and_lock_series(alice.clone(), vec![50], vec![50]);
            let nft_id_2 = help::create_nft_and_lock_series(alice.clone(), vec![50], vec![51]);
            let nft_id_3 = help::create_nft_and_lock_series(alice.clone(), vec![50], vec![52]);
            let nft_id_4 = help::create_nft_and_lock_series(alice.clone(), vec![50], vec![53]);
            let nft_id_5 = help::create_nft_and_lock_series(alice.clone(), vec![50], vec![54]);
            let mkt_id = help::create_mkp(dave.clone(), MPT::Private, 10, vec![0], vec![ALICE]);

            let caps = NFTCurrency::Caps(50);
            let tiime = NFTCurrency::Tiime(50);
            let comb = NFTCurrency::Combined(NFTCurrencyCombined::new(15, 5));
            assert_ok!(Marketplace::list(alice.clone(), nft_id_1, caps, None));
            assert_ok!(Marketplace::list(alice.clone(), nft_id_2, tiime, None));
            assert_ok!(Marketplace::list(alice.clone(), nft_id_3, comb, None));
            assert_ok!(Marketplace::list(alice.clone(), nft_id_4, comb, None));
            let ok = Marketplace::list(alice.clone(), nft_id_5, caps, Some(mkt_id));
            assert_ok!(ok);

            // Happy path CAPS
            let bob_before = Balances::free_balance(BOB);
            let alice_before = Balances::free_balance(ALICE);

            assert_ok!(Marketplace::buy(bob.clone(), nft_id_1, CAPS_ID));
            assert_eq!(NFTs::data(nft_id_1).unwrap().locked, false);
            assert_eq!(NFTs::data(nft_id_1).unwrap().owner, BOB);
            assert_eq!(Marketplace::nft_for_sale(nft_id_1), None);

            assert_eq!(Balances::free_balance(BOB), bob_before - 50);
            assert_eq!(Balances::free_balance(ALICE), alice_before + 50);

            // Happy path TIIME
            let bob_before = TiimeBalances::free_balance(BOB);
            let alice_before = TiimeBalances::free_balance(ALICE);

            assert_ok!(Marketplace::buy(bob.clone(), nft_id_2, TIIME_ID));
            assert_eq!(NFTs::data(nft_id_2).unwrap().locked, false);
            assert_eq!(NFTs::data(nft_id_2).unwrap().owner, BOB);
            assert_eq!(Marketplace::nft_for_sale(nft_id_2), None);

            assert_eq!(TiimeBalances::free_balance(BOB), bob_before - 50);
            assert_eq!(TiimeBalances::free_balance(ALICE), alice_before + 50);

            // Happy path COMBINED CAPS
            let bob_before = Balances::free_balance(BOB);
            let alice_before = Balances::free_balance(ALICE);

            assert_ok!(Marketplace::buy(bob.clone(), nft_id_3, CAPS_ID));
            assert_eq!(NFTs::data(nft_id_3).unwrap().locked, false);
            assert_eq!(NFTs::data(nft_id_3).unwrap().owner, BOB);
            assert_eq!(Marketplace::nft_for_sale(nft_id_3), None);

            assert_eq!(Balances::free_balance(BOB), bob_before - 15);
            assert_eq!(Balances::free_balance(ALICE), alice_before + 15);

            // Happy path COMBINED Tiime
            let bob_before = TiimeBalances::free_balance(BOB);
            let alice_before = TiimeBalances::free_balance(ALICE);

            assert_ok!(Marketplace::buy(bob.clone(), nft_id_4, TIIME_ID));
            assert_eq!(NFTs::data(nft_id_4).unwrap().locked, false);
            assert_eq!(NFTs::data(nft_id_4).unwrap().owner, BOB);
            assert_eq!(Marketplace::nft_for_sale(nft_id_4), None);

            assert_eq!(TiimeBalances::free_balance(BOB), bob_before - 5);
            assert_eq!(TiimeBalances::free_balance(ALICE), alice_before + 5);

            // Happy path PRIVATE (with commission fee)
            let bob_before = Balances::free_balance(BOB);
            let alice_before = Balances::free_balance(ALICE);
            let dave_before = Balances::free_balance(DAVE);

            assert_ok!(Marketplace::buy(bob.clone(), nft_id_5, CAPS_ID));
            assert_eq!(NFTs::data(nft_id_5).unwrap().locked, false);
            assert_eq!(NFTs::data(nft_id_5).unwrap().owner, BOB);
            assert_eq!(Marketplace::nft_for_sale(nft_id_5), None);

            assert_eq!(Balances::free_balance(BOB), bob_before - 50);
            assert_eq!(Balances::free_balance(ALICE), alice_before + 45);
            assert_eq!(Balances::free_balance(DAVE), dave_before + 5);
        })
}

#[test]
fn buy_unhappy() {
    ExtBuilder::default()
        .caps(vec![(ALICE, 100), (BOB, 100)])
        .tiime(vec![(ALICE, 100), (BOB, 100)])
        .build()
        .execute_with(|| {
            let alice: mock::Origin = RawOrigin::Signed(ALICE).into();
            let bob: mock::Origin = RawOrigin::Signed(BOB).into();

            let caps = NFTCurrency::Caps(5000);
            let tiime = NFTCurrency::Tiime(5000);
            let comb = NFTCurrency::Combined(NFTCurrencyCombined::new(5000, 5000));

            let nft_id_1 = help::create_nft_and_lock_series(alice.clone(), vec![50], vec![50]);
            let nft_id_2 = help::create_nft_and_lock_series(alice.clone(), vec![50], vec![51]);
            let nft_id_3 = help::create_nft_and_lock_series(alice.clone(), vec![50], vec![52]);

            assert_ok!(Marketplace::list(alice.clone(), nft_id_1, caps, None));
            assert_ok!(Marketplace::list(alice.clone(), nft_id_2, tiime, None));
            assert_ok!(Marketplace::list(alice.clone(), nft_id_3, comb, None));

            // Unhappy nft not on sale
            let ok = Marketplace::buy(bob.clone(), 1001, CAPS_ID);
            assert_noop!(ok, Error::<Test>::NftNotForSale);

            // Unhappy not enough caps
            let ok = Marketplace::buy(bob.clone(), nft_id_1, CAPS_ID);
            assert_noop!(ok, BalanceError::<Test>::InsufficientBalance);

            // Unhappy not enough tiime
            let ok = Marketplace::buy(bob.clone(), nft_id_2, TIIME_ID);
            assert_noop!(ok, BalanceError::<Test, Instance1>::InsufficientBalance);

            // Unhappy not enough combined caps
            let ok = Marketplace::buy(bob.clone(), nft_id_3, CAPS_ID);
            assert_noop!(ok, BalanceError::<Test>::InsufficientBalance);

            // Unhappy not enough combined tiime
            let ok = Marketplace::buy(bob.clone(), nft_id_3, TIIME_ID);
            assert_noop!(ok, BalanceError::<Test, Instance1>::InsufficientBalance);

            // Unhappy wrong currency used (expects CAPS)
            let ok = Marketplace::buy(bob.clone(), nft_id_1, TIIME_ID);
            assert_noop!(ok, Error::<Test>::WrongCurrencyUsed);

            // Unhappy wrong currency used (expects TIIME)
            let ok = Marketplace::buy(bob.clone(), nft_id_2, CAPS_ID);
            assert_noop!(ok, Error::<Test>::WrongCurrencyUsed);
        })
}

#[test]
fn create_happy() {
    ExtBuilder::default()
        .caps(vec![(ALICE, 10000)])
        .build()
        .execute_with(|| {
            let alice: mock::Origin = RawOrigin::Signed(ALICE).into();

            assert_eq!(Marketplace::marketplace_id_generator(), 0);
            assert_eq!(Marketplace::marketplaces(1), None);
            let balance = Balances::free_balance(ALICE);
            let fee = 25;
            let name = vec![50];
            let kind = MPT::Public;
            let uri = Some(vec![65]);
            let logo_uri = Some(vec![66]);
            let info = MarketplaceInformation::new(
                kind,
                fee,
                ALICE,
                vec![],
                name.clone(),
                uri.clone(),
                logo_uri.clone(),
            );

            // Happy path
            assert_ok!(Marketplace::create(
                alice.clone(),
                kind,
                fee,
                name,
                uri.clone(),
                logo_uri.clone()
            ));
            assert_eq!(Marketplace::marketplace_id_generator(), 1);
            assert_eq!(Marketplace::marketplaces(1), Some(info));
            assert_eq!(
                Balances::free_balance(ALICE),
                balance - Marketplace::marketplace_mint_fee()
            );
        })
}

#[test]
fn create_unhappy() {
    ExtBuilder::default()
        .caps(vec![(ALICE, 5)])
        .build()
        .execute_with(|| {
            let alice: mock::Origin = RawOrigin::Signed(ALICE).into();
            let normal_uri: Option<URI> = Some(vec![66]);
            let too_short_uri: Option<URI> = Some(vec![]);
            let too_long_uri: Option<URI> = Some([0; 1001].to_vec());

            // Unhappy invalid commission fee
            let ok = Marketplace::create(
                alice.clone(),
                MPT::Public,
                101,
                vec![50],
                normal_uri.clone(),
                normal_uri.clone(),
            );
            assert_noop!(ok, Error::<Test>::InvalidCommissionFeeValue);

            // Unhappy too short name
            let ok = Marketplace::create(
                alice.clone(),
                MPT::Public,
                0,
                vec![],
                normal_uri.clone(),
                normal_uri.clone(),
            );
            assert_noop!(ok, Error::<Test>::TooShortMarketplaceName);

            // Unhappy too long name
            let ok = Marketplace::create(
                alice.clone(),
                MPT::Public,
                0,
                vec![1, 2, 3, 4, 5, 6],
                normal_uri.clone(),
                normal_uri.clone(),
            );
            assert_noop!(ok, Error::<Test>::TooLongMarketplaceName);

            // Unhappy not enough funds
            let ok = Marketplace::create(
                alice.clone(),
                MPT::Public,
                5,
                vec![50],
                normal_uri.clone(),
                normal_uri.clone(),
            );
            assert_noop!(ok, BalanceError::<Test>::InsufficientBalance);

            // Unhappy too short uri
            let ok = Marketplace::create(
                alice.clone(),
                MPT::Public,
                0,
                vec![50],
                too_short_uri.clone(),
                normal_uri.clone(),
            );
            assert_noop!(ok, Error::<Test>::TooShortMarketplaceUri);

            // Unhappy too long uri
            let ok = Marketplace::create(
                alice.clone(),
                MPT::Public,
                0,
                vec![50],
                too_long_uri.clone(),
                normal_uri.clone(),
            );
            assert_noop!(ok, Error::<Test>::TooLongMarketplaceUri);

            // Unhappy too short logo uri
            let ok = Marketplace::create(
                alice.clone(),
                MPT::Public,
                0,
                vec![50],
                normal_uri.clone(),
                too_short_uri,
            );
            assert_noop!(ok, Error::<Test>::TooShortMarketplaceLogoUri);

            // Unhappy too long logo uri
            let ok = Marketplace::create(
                alice.clone(),
                MPT::Public,
                0,
                vec![50],
                normal_uri,
                too_long_uri,
            );
            assert_noop!(ok, Error::<Test>::TooLongMarketplaceLogoUri);
        })
}

#[test]
fn add_account_to_allow_list_happy() {
    ExtBuilder::default()
        .caps(vec![(ALICE, 1000)])
        .build()
        .execute_with(|| {
            let alice: mock::Origin = RawOrigin::Signed(ALICE).into();

            // Happy path
            let list = vec![];
            let mkp_1 = help::create_mkp(alice.clone(), MPT::Private, 0, vec![50], list.clone());
            assert_eq!(Marketplace::marketplaces(mkp_1).unwrap().allow_list, list);

            let ok = Marketplace::add_account_to_allow_list(alice.clone(), mkp_1, BOB);
            assert_ok!(ok);
            let list = vec![BOB];
            assert_eq!(Marketplace::marketplaces(mkp_1).unwrap().allow_list, list);
        })
}

#[test]
fn add_account_to_allow_list_unhappy() {
    ExtBuilder::default()
        .caps(vec![(BOB, 1000), (DAVE, 1000)])
        .build()
        .execute_with(|| {
            let bob: mock::Origin = RawOrigin::Signed(BOB).into();

            // Unhappy unknown marketplace
            let ok = Marketplace::add_account_to_allow_list(bob.clone(), 1001, DAVE);
            assert_noop!(ok, Error::<Test>::UnknownMarketplace);

            // Unhappy not marketplace owner
            let ok = Marketplace::add_account_to_allow_list(bob.clone(), 0, DAVE);
            assert_noop!(ok, Error::<Test>::NotMarketplaceOwner);

            // Unhappy unsupported marketplace type
            let mkp_id = help::create_mkp(bob.clone(), MPT::Public, 0, vec![50], vec![]);
            let ok = Marketplace::add_account_to_allow_list(bob.clone(), mkp_id, DAVE);
            assert_noop!(ok, Error::<Test>::UnsupportedMarketplace);
        })
}

#[test]
fn remove_account_from_allow_list_happy() {
    ExtBuilder::default()
        .caps(vec![(ALICE, 1000)])
        .build()
        .execute_with(|| {
            let alice: mock::Origin = RawOrigin::Signed(ALICE).into();

            // Happy path
            let list = vec![BOB];
            let mkp_id = help::create_mkp(alice.clone(), MPT::Private, 0, vec![50], list.clone());
            assert_eq!(Marketplace::marketplaces(mkp_id).unwrap().allow_list, list);

            let ok = Marketplace::remove_account_from_allow_list(alice.clone(), mkp_id, BOB);
            assert_ok!(ok);
            let list: Vec<u64> = vec![];
            assert_eq!(Marketplace::marketplaces(mkp_id).unwrap().allow_list, list);
        })
}

#[test]
fn remove_account_from_allow_list_unhappy() {
    ExtBuilder::default()
        .caps(vec![(BOB, 1000), (DAVE, 1000)])
        .build()
        .execute_with(|| {
            let bob: mock::Origin = RawOrigin::Signed(BOB).into();

            // Unhappy unknown marketplace
            let ok = Marketplace::remove_account_from_allow_list(bob.clone(), 1001, DAVE);
            assert_noop!(ok, Error::<Test>::UnknownMarketplace);

            // Unhappy not marketplace owner
            let ok = Marketplace::remove_account_from_allow_list(bob.clone(), 0, DAVE);
            assert_noop!(ok, Error::<Test>::NotMarketplaceOwner);

            // Unhappy unsupported marketplace type
            let mkp_id = help::create_mkp(bob.clone(), MPT::Public, 0, vec![50], vec![]);
            let ok = Marketplace::remove_account_from_allow_list(bob.clone(), mkp_id, DAVE);
            assert_noop!(ok, Error::<Test>::UnsupportedMarketplace);
        })
}

#[test]
fn set_owner_happy() {
    ExtBuilder::default()
        .caps(vec![(ALICE, 1000), (BOB, 1000)])
        .build()
        .execute_with(|| {
            let alice: mock::Origin = RawOrigin::Signed(ALICE).into();

            // Happy path
            let mkp_id = help::create_mkp(alice.clone(), MPT::Private, 0, vec![50], vec![]);
            assert_ok!(Marketplace::set_owner(alice.clone(), mkp_id, BOB));
            assert_eq!(Marketplace::marketplaces(mkp_id).unwrap().owner, BOB);
        })
}

#[test]
fn set_owner_unhappy() {
    ExtBuilder::default()
        .caps(vec![(BOB, 1000)])
        .build()
        .execute_with(|| {
            let bob: mock::Origin = RawOrigin::Signed(BOB).into();

            // Unhappy unknown marketplace
            let ok = Marketplace::set_owner(bob.clone(), 1001, DAVE);
            assert_noop!(ok, Error::<Test>::UnknownMarketplace);

            // Unhappy not marketplace owner
            let ok = Marketplace::set_owner(bob.clone(), 0, DAVE);
            assert_noop!(ok, Error::<Test>::NotMarketplaceOwner);
        })
}

#[test]
fn set_market_type_happy() {
    ExtBuilder::default()
        .caps(vec![(ALICE, 1000), (BOB, 1000)])
        .build()
        .execute_with(|| {
            let alice: mock::Origin = RawOrigin::Signed(ALICE).into();

            let kind = MPT::Public;
            let mkp_id = help::create_mkp(alice.clone(), MPT::Public, 0, vec![50], vec![]);
            assert_eq!(Marketplace::marketplaces(mkp_id).unwrap().kind, kind);

            // Happy path Public to Private
            let kind = MPT::Private;
            assert_ok!(Marketplace::set_market_type(alice.clone(), mkp_id, kind));
            assert_eq!(Marketplace::marketplaces(mkp_id).unwrap().kind, kind);

            // Happy path Private to Public
            let kind = MPT::Public;
            assert_ok!(Marketplace::set_market_type(alice.clone(), mkp_id, kind));
            assert_eq!(Marketplace::marketplaces(mkp_id).unwrap().kind, kind);
        })
}

#[test]
fn set_market_type_unhappy() {
    ExtBuilder::default()
        .caps(vec![(BOB, 1000)])
        .build()
        .execute_with(|| {
            let bob: mock::Origin = RawOrigin::Signed(BOB).into();

            let kind = MPT::Public;

            // Unhappy unknown marketplace
            let ok = Marketplace::set_market_type(bob.clone(), 1001, kind);
            assert_noop!(ok, Error::<Test>::UnknownMarketplace);

            // Unhappy not marketplace owner
            let ok = Marketplace::set_market_type(bob.clone(), 0, kind);
            assert_noop!(ok, Error::<Test>::NotMarketplaceOwner);
        })
}

#[test]
fn set_name_happy() {
    ExtBuilder::default()
        .caps(vec![(ALICE, 1000), (BOB, 1000)])
        .build()
        .execute_with(|| {
            let alice: mock::Origin = RawOrigin::Signed(ALICE).into();

            // Happy path
            let name = vec![50];
            let mkp_id = help::create_mkp(alice.clone(), MPT::Private, 0, name.clone(), vec![]);
            assert_eq!(Marketplace::marketplaces(mkp_id).unwrap().name, name);

            let name = vec![51];
            assert_ok!(Marketplace::set_name(alice.clone(), mkp_id, name.clone()));
            assert_eq!(Marketplace::marketplaces(mkp_id).unwrap().name, name);
        })
}

#[test]
fn set_name_unhappy() {
    ExtBuilder::default()
        .caps(vec![(BOB, 1000)])
        .build()
        .execute_with(|| {
            let bob: mock::Origin = RawOrigin::Signed(BOB).into();

            // Unhappy too short name
            let ok = Marketplace::set_name(bob.clone(), 0, vec![]);
            assert_noop!(ok, Error::<Test>::TooShortMarketplaceName);

            // Unhappy too long name
            let ok = Marketplace::set_name(bob.clone(), 0, vec![1, 2, 3, 4, 5, 6]);
            assert_noop!(ok, Error::<Test>::TooLongMarketplaceName);

            // Unhappy unknown marketplace
            let ok = Marketplace::set_name(bob.clone(), 1001, vec![51]);
            assert_noop!(ok, Error::<Test>::UnknownMarketplace);

            // Unhappy not marketplace owner
            let ok = Marketplace::set_name(bob.clone(), 0, vec![51]);
            assert_noop!(ok, Error::<Test>::NotMarketplaceOwner);
        })
}

#[test]
fn set_marketplace_mint_fee_happy() {
    ExtBuilder::default().build().execute_with(|| {
        // Happy path
        let old_mint_fee = Marketplace::marketplace_mint_fee();
        let new_mint_fee = 654u64;
        assert_eq!(Marketplace::marketplace_mint_fee(), old_mint_fee);

        let ok = Marketplace::set_marketplace_mint_fee(mock::Origin::root(), new_mint_fee);
        assert_ok!(ok);
        assert_eq!(Marketplace::marketplace_mint_fee(), new_mint_fee);
    })
}

#[test]
fn set_marketplace_mint_fee_unhappy() {
    ExtBuilder::default()
        .caps(vec![(ALICE, 10000)])
        .build()
        .execute_with(|| {
            let alice: mock::Origin = RawOrigin::Signed(ALICE).into();

            // Unhappy non root user tries to modify the mint fee
            let ok = Marketplace::set_marketplace_mint_fee(alice.clone(), 654);
            assert_noop!(ok, BadOrigin);
        })
}

#[test]
fn set_commission_fee_happy() {
    ExtBuilder::default()
        .caps(vec![(ALICE, 1000)])
        .build()
        .execute_with(|| {
            let alice: mock::Origin = RawOrigin::Signed(ALICE).into();

            let fee = 10;
            let id = help::create_mkp(alice.clone(), MPT::Public, fee, vec![50], vec![]);
            assert_eq!(Marketplace::marketplaces(id).unwrap().commission_fee, fee);

            // Happy path
            let fee = 15;
            assert_ok!(Marketplace::set_commission_fee(alice.clone(), id, fee));
            assert_eq!(Marketplace::marketplaces(id).unwrap().commission_fee, fee);
        })
}

#[test]
fn set_commission_fee_unhappy() {
    ExtBuilder::default()
        .caps(vec![(BOB, 1000)])
        .build()
        .execute_with(|| {
            let bob: mock::Origin = RawOrigin::Signed(BOB).into();

            // Unhappy commission fee is more than 100
            let ok = Marketplace::set_commission_fee(bob.clone(), 0, 101);
            assert_noop!(ok, Error::<Test>::InvalidCommissionFeeValue);

            // Unhappy unknown marketplace
            let ok = Marketplace::set_commission_fee(bob.clone(), 1001, 15);
            assert_noop!(ok, Error::<Test>::UnknownMarketplace);

            // Unhappy not marketplace owner
            let ok = Marketplace::set_commission_fee(bob.clone(), 0, 15);
            assert_noop!(ok, Error::<Test>::NotMarketplaceOwner);
        })
}

#[test]
fn update_uri_happy() {
    ExtBuilder::default()
        .caps(vec![(ALICE, 10000)])
        .build()
        .execute_with(|| {
            let alice: mock::Origin = RawOrigin::Signed(ALICE).into();

            let fee = 25;
            let name = vec![50];
            let kind = MPT::Public;
            let uri = Some(vec![66]);
            let updated_uri = Some(vec![67]);

            let updated_info: MarketplaceInformation<Test> = MarketplaceInformation::new(
                kind,
                fee,
                ALICE,
                vec![],
                name.clone(),
                updated_uri.clone(),
                uri.clone(),
            );

            assert_ok!(Marketplace::create(
                alice.clone(),
                kind.clone(),
                fee,
                name.clone(),
                uri.clone(),
                uri.clone()
            ));
            assert_ne!(
                Marketplace::marketplaces(1).unwrap().uri,
                updated_uri.clone()
            );
            assert_ok!(Marketplace::update_uri(
                alice.clone(),
                1,
                updated_uri.unwrap()
            ));
            assert_eq!(Marketplace::marketplaces(1), Some(updated_info));
        })
}

#[test]
fn update_uri_unhappy() {
    ExtBuilder::default()
        .caps(vec![(ALICE, 10000)])
        .build()
        .execute_with(|| {
            let alice: mock::Origin = RawOrigin::Signed(ALICE).into();

            let fee = 25;
            let name = vec![50];
            let kind = MPT::Public;
            let uri = Some(vec![66]);
            let raw_too_short_uri: URI = vec![];
            let raw_too_long_uri: URI = [0; 1001].to_vec();

            assert_ok!(Marketplace::create(
                alice.clone(),
                kind.clone(),
                fee,
                name.clone(),
                uri.clone(),
                uri.clone()
            ));

            let nok = Marketplace::update_uri(alice.clone(), 1, raw_too_short_uri);
            assert_noop!(nok, Error::<Test>::TooShortMarketplaceUri);
            let nok = Marketplace::update_uri(alice, 1, raw_too_long_uri);
            assert_noop!(nok, Error::<Test>::TooLongMarketplaceUri);
        })
}

#[test]
fn update_logo_uri_happy() {
    ExtBuilder::default()
        .caps(vec![(ALICE, 10000)])
        .build()
        .execute_with(|| {
            let alice: mock::Origin = RawOrigin::Signed(ALICE).into();

            let fee = 25;
            let name = vec![50];
            let kind = MPT::Public;
            let uri = Some(vec![66]);
            let updated_uri = Some(vec![67]);

            let updated_info = MarketplaceInformation::new(
                kind,
                fee,
                ALICE,
                vec![],
                name.clone(),
                uri.clone(),
                updated_uri.clone(),
            );

            assert_ok!(Marketplace::create(
                alice.clone(),
                kind.clone(),
                fee,
                name.clone(),
                uri.clone(),
                uri.clone()
            ));
            assert_ne!(
                Marketplace::marketplaces(1).unwrap().uri,
                updated_uri.clone()
            );

            assert_ok!(Marketplace::update_logo_uri(
                alice.clone(),
                1,
                updated_uri.unwrap()
            ));
            assert_eq!(Marketplace::marketplaces(1), Some(updated_info));
        })
}

#[test]
fn update_logo_uri_unhappy() {
    ExtBuilder::default()
        .caps(vec![(ALICE, 10000)])
        .build()
        .execute_with(|| {
            let alice: mock::Origin = RawOrigin::Signed(ALICE).into();

            let fee = 25;
            let name = vec![50];
            let kind = MPT::Public;
            let uri = Some(vec![66]);
            let raw_too_short_uri: URI = vec![];
            let raw_too_long_uri: URI = [0; 1001].to_vec();

            assert_ok!(Marketplace::create(
                alice.clone(),
                kind.clone(),
                fee,
                name.clone(),
                uri.clone(),
                uri.clone()
            ));

            let nok = Marketplace::update_logo_uri(alice.clone(), 1, raw_too_short_uri);
            assert_noop!(nok, Error::<Test>::TooShortMarketplaceLogoUri);
            let nok = Marketplace::update_logo_uri(alice, 1, raw_too_long_uri);
            assert_noop!(nok, Error::<Test>::TooLongMarketplaceLogoUri);
        })
}