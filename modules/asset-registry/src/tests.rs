// This file is part of Acala.

// Copyright (C) 2020-2021 Acala Foundation.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Unit tests for asset registry module.

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::{
	alice, deploy_contracts, erc20_address, erc20_address_not_exists, AssetRegistry, CouncilAccount, Event, ExtBuilder,
	Origin, Runtime, System,
};
use orml_utilities::with_transaction_result;
use primitives::TokenSymbol;
use sp_core::H160;
use std::str::FromStr;

#[test]
fn versioned_multi_location_convert_work() {
	ExtBuilder::default().build().execute_with(|| {
		// v0
		let v0_location = VersionedMultiLocation::V0(xcm::v0::MultiLocation::X1(xcm::v0::Junction::Parachain(1000)));
		let location: MultiLocation = v0_location.try_into().unwrap();
		assert_eq!(
			location,
			MultiLocation {
				parents: 0,
				interior: xcm::v1::Junctions::X1(xcm::v1::Junction::Parachain(1000))
			}
		);

		// v1
		let v1_location = VersionedMultiLocation::V1(MultiLocation {
			parents: 0,
			interior: xcm::v1::Junctions::X1(xcm::v1::Junction::Parachain(1000)),
		});
		let location: MultiLocation = v1_location.try_into().unwrap();
		assert_eq!(
			location,
			MultiLocation {
				parents: 0,
				interior: xcm::v1::Junctions::X1(xcm::v1::Junction::Parachain(1000))
			}
		);

		// handle all of VersionedMultiLocation
		assert!(match location.into() {
			VersionedMultiLocation::V0 { .. } | VersionedMultiLocation::V1 { .. } => true,
		});
	});
}

#[test]
fn register_foreign_asset_work() {
	ExtBuilder::default().build().execute_with(|| {
		let v0_location = VersionedMultiLocation::V0(xcm::v0::MultiLocation::X1(xcm::v0::Junction::Parachain(1000)));

		assert_ok!(AssetRegistry::register_foreign_asset(
			Origin::signed(CouncilAccount::get()),
			Box::new(v0_location.clone()),
			Box::new(AssetMetadata {
				name: b"Token Name".to_vec(),
				symbol: b"TN".to_vec(),
				decimals: 12,
				minimal_balance: 1,
			})
		));

		let location: MultiLocation = v0_location.try_into().unwrap();
		System::assert_last_event(Event::AssetRegistry(crate::Event::ForeignAssetRegistered(
			0,
			location.clone(),
			AssetMetadata {
				name: b"Token Name".to_vec(),
				symbol: b"TN".to_vec(),
				decimals: 12,
				minimal_balance: 1,
			},
		)));

		assert_eq!(ForeignAssetLocations::<Runtime>::get(0), Some(location.clone()));
		assert_eq!(
			AssetMetadatas::<Runtime>::get(0),
			Some(AssetMetadata {
				name: b"Token Name".to_vec(),
				symbol: b"TN".to_vec(),
				decimals: 12,
				minimal_balance: 1,
			})
		);
		assert_eq!(
			LocationToCurrencyIds::<Runtime>::get(location),
			Some(CurrencyId::ForeignAsset(0))
		);
	});
}

#[test]
fn register_foreign_asset_should_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		let v0_location = VersionedMultiLocation::V0(xcm::v0::MultiLocation::X1(xcm::v0::Junction::Parachain(1000)));
		assert_ok!(AssetRegistry::register_foreign_asset(
			Origin::signed(CouncilAccount::get()),
			Box::new(v0_location.clone()),
			Box::new(AssetMetadata {
				name: b"Token Name".to_vec(),
				symbol: b"TN".to_vec(),
				decimals: 12,
				minimal_balance: 1,
			})
		));

		assert_noop!(
			AssetRegistry::register_foreign_asset(
				Origin::signed(CouncilAccount::get()),
				Box::new(v0_location.clone()),
				Box::new(AssetMetadata {
					name: b"Token Name".to_vec(),
					symbol: b"TN".to_vec(),
					decimals: 12,
					minimal_balance: 1,
				})
			),
			Error::<Runtime>::MultiLocationExisted
		);

		NextForeignAssetId::<Runtime>::set(u16::MAX);
		assert_noop!(
			AssetRegistry::register_foreign_asset(
				Origin::signed(CouncilAccount::get()),
				Box::new(VersionedMultiLocation::V0(xcm::v0::MultiLocation::X1(
					xcm::v0::Junction::Parachain(1000)
				))),
				Box::new(AssetMetadata {
					name: b"Token Name".to_vec(),
					symbol: b"TN".to_vec(),
					decimals: 12,
					minimal_balance: 1,
				})
			),
			ArithmeticError::Overflow
		);
	});
}

#[test]
fn update_foreign_asset_work() {
	ExtBuilder::default().build().execute_with(|| {
		let v0_location = VersionedMultiLocation::V0(xcm::v0::MultiLocation::X1(xcm::v0::Junction::Parachain(1000)));

		assert_ok!(AssetRegistry::register_foreign_asset(
			Origin::signed(CouncilAccount::get()),
			Box::new(v0_location.clone()),
			Box::new(AssetMetadata {
				name: b"Token Name".to_vec(),
				symbol: b"TN".to_vec(),
				decimals: 12,
				minimal_balance: 1,
			})
		));

		assert_ok!(AssetRegistry::update_foreign_asset(
			Origin::signed(CouncilAccount::get()),
			0,
			Box::new(v0_location.clone()),
			Box::new(AssetMetadata {
				name: b"New Token Name".to_vec(),
				symbol: b"NTN".to_vec(),
				decimals: 13,
				minimal_balance: 2,
			})
		));

		let location: MultiLocation = v0_location.try_into().unwrap();
		System::assert_last_event(Event::AssetRegistry(crate::Event::ForeignAssetUpdated(
			location.clone(),
			AssetMetadata {
				name: b"New Token Name".to_vec(),
				symbol: b"NTN".to_vec(),
				decimals: 13,
				minimal_balance: 2,
			},
		)));

		assert_eq!(
			AssetMetadatas::<Runtime>::get(0),
			Some(AssetMetadata {
				name: b"New Token Name".to_vec(),
				symbol: b"NTN".to_vec(),
				decimals: 13,
				minimal_balance: 2,
			})
		);
		assert_eq!(ForeignAssetLocations::<Runtime>::get(0), Some(location.clone()));
		assert_eq!(
			LocationToCurrencyIds::<Runtime>::get(location.clone()),
			Some(CurrencyId::ForeignAsset(0))
		);

		// modify location
		let new_location = VersionedMultiLocation::V0(xcm::v0::MultiLocation::X1(xcm::v0::Junction::Parachain(2000)));
		assert_ok!(AssetRegistry::update_foreign_asset(
			Origin::signed(CouncilAccount::get()),
			0,
			Box::new(new_location.clone()),
			Box::new(AssetMetadata {
				name: b"New Token Name".to_vec(),
				symbol: b"NTN".to_vec(),
				decimals: 13,
				minimal_balance: 2,
			})
		));
		assert_eq!(
			AssetMetadatas::<Runtime>::get(0),
			Some(AssetMetadata {
				name: b"New Token Name".to_vec(),
				symbol: b"NTN".to_vec(),
				decimals: 13,
				minimal_balance: 2,
			})
		);
		let new_location: MultiLocation = new_location.try_into().unwrap();
		assert_eq!(ForeignAssetLocations::<Runtime>::get(0), Some(new_location.clone()));
		assert_eq!(LocationToCurrencyIds::<Runtime>::get(location), None);
		assert_eq!(
			LocationToCurrencyIds::<Runtime>::get(new_location),
			Some(CurrencyId::ForeignAsset(0))
		);
	});
}

#[test]
fn update_foreign_asset_should_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		let v0_location = VersionedMultiLocation::V0(xcm::v0::MultiLocation::X1(xcm::v0::Junction::Parachain(1000)));

		assert_noop!(
			AssetRegistry::update_foreign_asset(
				Origin::signed(CouncilAccount::get()),
				0,
				Box::new(v0_location.clone()),
				Box::new(AssetMetadata {
					name: b"New Token Name".to_vec(),
					symbol: b"NTN".to_vec(),
					decimals: 13,
					minimal_balance: 2,
				})
			),
			Error::<Runtime>::ForeignAssetIdNotExists
		);

		assert_ok!(AssetRegistry::register_foreign_asset(
			Origin::signed(CouncilAccount::get()),
			Box::new(v0_location.clone()),
			Box::new(AssetMetadata {
				name: b"Token Name".to_vec(),
				symbol: b"TN".to_vec(),
				decimals: 12,
				minimal_balance: 1,
			})
		));

		assert_ok!(AssetRegistry::update_foreign_asset(
			Origin::signed(CouncilAccount::get()),
			0,
			Box::new(v0_location.clone()),
			Box::new(AssetMetadata {
				name: b"New Token Name".to_vec(),
				symbol: b"NTN".to_vec(),
				decimals: 13,
				minimal_balance: 2,
			})
		));

		// existed location
		let new_location = VersionedMultiLocation::V0(xcm::v0::MultiLocation::X1(xcm::v0::Junction::Parachain(2000)));
		assert_ok!(AssetRegistry::register_foreign_asset(
			Origin::signed(CouncilAccount::get()),
			Box::new(new_location.clone()),
			Box::new(AssetMetadata {
				name: b"Token Name".to_vec(),
				symbol: b"TN".to_vec(),
				decimals: 12,
				minimal_balance: 1,
			})
		));
		assert_noop!(
			AssetRegistry::update_foreign_asset(
				Origin::signed(CouncilAccount::get()),
				0,
				Box::new(new_location.clone()),
				Box::new(AssetMetadata {
					name: b"New Token Name".to_vec(),
					symbol: b"NTN".to_vec(),
					decimals: 13,
					minimal_balance: 2,
				})
			),
			Error::<Runtime>::MultiLocationExisted
		);
	});
}

#[test]
fn set_erc20_mapping_works() {
	ExtBuilder::default()
		.balances(vec![(alice(), 1_000_000_000_000)])
		.build()
		.execute_with(|| {
			deploy_contracts();
			assert_ok!(with_transaction_result(|| -> DispatchResult {
				EvmErc20InfoMapping::<Runtime>::set_erc20_mapping(erc20_address())
			}));

			assert_ok!(with_transaction_result(|| -> DispatchResult {
				EvmErc20InfoMapping::<Runtime>::set_erc20_mapping(erc20_address())
			}));

			assert_noop!(
				with_transaction_result(|| -> DispatchResult {
					let mut addr = erc20_address();
					let addr = addr.as_bytes_mut();
					addr[19] += 1;
					EvmErc20InfoMapping::<Runtime>::set_erc20_mapping(EvmAddress::from_slice(addr))
				}),
				Error::<Runtime>::CurrencyIdExisted,
			);

			assert_noop!(
				with_transaction_result(|| -> DispatchResult {
					EvmErc20InfoMapping::<Runtime>::set_erc20_mapping(erc20_address_not_exists())
				}),
				module_evm_bridge::Error::<Runtime>::InvalidReturnValue,
			);
		});
}

#[test]
fn get_evm_address_works() {
	ExtBuilder::default()
		.balances(vec![(alice(), 1_000_000_000_000)])
		.build()
		.execute_with(|| {
			deploy_contracts();
			assert_ok!(with_transaction_result(|| -> DispatchResult {
				EvmErc20InfoMapping::<Runtime>::set_erc20_mapping(erc20_address())
			}));
			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::get_evm_address(DexShare::Erc20(erc20_address()).into()),
				Some(erc20_address())
			);

			assert_eq!(EvmErc20InfoMapping::<Runtime>::get_evm_address(u32::default()), None);
		});
}

#[test]
fn name_works() {
	ExtBuilder::default()
		.balances(vec![(alice(), 1_000_000_000_000)])
		.build()
		.execute_with(|| {
			deploy_contracts();
			assert_ok!(with_transaction_result(|| -> DispatchResult {
				EvmErc20InfoMapping::<Runtime>::set_erc20_mapping(erc20_address())
			}));
			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::name(CurrencyId::Token(TokenSymbol::ACA)),
				Some(b"Acala".to_vec())
			);
			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::name(CurrencyId::Erc20(erc20_address())),
				Some(b"long string name, long string name, long string name, long string name, long string name"[..32].to_vec())
			);

			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::name(CurrencyId::Erc20(erc20_address_not_exists())),
				None
			);

			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::name(CurrencyId::DexShare(DexShare::Token(TokenSymbol::ACA), DexShare::Token(TokenSymbol::AUSD))),
				Some(b"LP Acala - Acala Dollar".to_vec())
			);

			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::name(CurrencyId::DexShare(DexShare::Erc20(erc20_address()), DexShare::Token(TokenSymbol::AUSD))),
				Some(b"LP long string name, long string name, long string name, long string name, long string name - Acala Dollar"[..32].to_vec())
			);

			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::name(CurrencyId::DexShare(DexShare::Erc20(erc20_address()), DexShare::Erc20(erc20_address()))),
				Some(b"LP long string name, long string name, long string name, long string name, long string name - long string name, long string name, long string name, long string name, long string name"[..32].to_vec())
			);

			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::name(CurrencyId::DexShare(DexShare::Token(TokenSymbol::ACA), DexShare::Erc20(erc20_address_not_exists()))),
				None
			);

			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::name(CurrencyId::DexShare(DexShare::Erc20(erc20_address()), DexShare::Erc20(erc20_address_not_exists()))),
				None
			);
		});
}

#[test]
fn symbol_works() {
	ExtBuilder::default()
		.balances(vec![(alice(), 1_000_000_000_000)])
		.build()
		.execute_with(|| {
			deploy_contracts();
			assert_ok!(with_transaction_result(|| -> DispatchResult {
				EvmErc20InfoMapping::<Runtime>::set_erc20_mapping(erc20_address())
			}));
			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::symbol(CurrencyId::Token(TokenSymbol::ACA)),
				Some(b"ACA".to_vec())
			);
			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::symbol(CurrencyId::Erc20(erc20_address())),
				Some(b"TestToken".to_vec())
			);

			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::symbol(CurrencyId::Erc20(erc20_address_not_exists())),
				None
			);

			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::symbol(CurrencyId::DexShare(
					DexShare::Token(TokenSymbol::ACA),
					DexShare::Token(TokenSymbol::AUSD)
				)),
				Some(b"LP_ACA_AUSD".to_vec())
			);

			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::symbol(CurrencyId::DexShare(
					DexShare::Erc20(erc20_address()),
					DexShare::Token(TokenSymbol::AUSD)
				)),
				Some(b"LP_TestToken_AUSD".to_vec())
			);

			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::symbol(CurrencyId::DexShare(
					DexShare::Erc20(erc20_address()),
					DexShare::Erc20(erc20_address())
				)),
				Some(b"LP_TestToken_TestToken".to_vec())
			);

			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::symbol(CurrencyId::DexShare(
					DexShare::Token(TokenSymbol::ACA),
					DexShare::Erc20(erc20_address_not_exists())
				)),
				None
			);

			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::symbol(CurrencyId::DexShare(
					DexShare::Erc20(erc20_address()),
					DexShare::Erc20(erc20_address_not_exists())
				)),
				None
			);
		});
}

#[test]
fn decimals_works() {
	ExtBuilder::default()
		.balances(vec![(alice(), 1_000_000_000_000)])
		.build()
		.execute_with(|| {
			deploy_contracts();
			assert_ok!(with_transaction_result(|| -> DispatchResult {
				EvmErc20InfoMapping::<Runtime>::set_erc20_mapping(erc20_address())
			}));
			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::decimals(CurrencyId::Token(TokenSymbol::ACA)),
				Some(12)
			);
			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::decimals(CurrencyId::Erc20(erc20_address())),
				Some(17)
			);

			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::decimals(CurrencyId::Erc20(erc20_address_not_exists())),
				None
			);

			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::decimals(CurrencyId::DexShare(
					DexShare::Token(TokenSymbol::ACA),
					DexShare::Token(TokenSymbol::AUSD)
				)),
				Some(12)
			);

			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::decimals(CurrencyId::DexShare(
					DexShare::Erc20(erc20_address()),
					DexShare::Token(TokenSymbol::AUSD)
				)),
				Some(17)
			);

			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::decimals(CurrencyId::DexShare(
					DexShare::Erc20(erc20_address()),
					DexShare::Erc20(erc20_address())
				)),
				Some(17)
			);

			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::decimals(CurrencyId::DexShare(
					DexShare::Erc20(erc20_address()),
					DexShare::Erc20(erc20_address_not_exists())
				)),
				Some(17)
			);
		});
}

#[test]
fn encode_evm_address_works() {
	ExtBuilder::default()
		.balances(vec![(alice(), 1_000_000_000_000)])
		.build()
		.execute_with(|| {
			deploy_contracts();
			assert_ok!(with_transaction_result(|| -> DispatchResult {
				EvmErc20InfoMapping::<Runtime>::set_erc20_mapping(erc20_address())
			}));

			// Token
			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::encode_evm_address(CurrencyId::Token(TokenSymbol::ACA)),
				H160::from_str("0x0000000000000000000100000000000000000000").ok()
			);

			// Erc20
			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::encode_evm_address(CurrencyId::Erc20(erc20_address())),
				Some(erc20_address())
			);

			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::encode_evm_address(CurrencyId::Erc20(erc20_address_not_exists())),
				Some(erc20_address_not_exists())
			);

			// DexShare
			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::encode_evm_address(CurrencyId::DexShare(
					DexShare::Token(TokenSymbol::ACA),
					DexShare::Token(TokenSymbol::AUSD)
				)),
				H160::from_str("0x0000000000000000000200000000000000000001").ok()
			);

			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::encode_evm_address(CurrencyId::DexShare(
					DexShare::Erc20(erc20_address()),
					DexShare::Token(TokenSymbol::AUSD)
				)),
				H160::from_str("0x00000000000000000002015dddfce50000000001").ok()
			);

			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::encode_evm_address(CurrencyId::DexShare(
					DexShare::Token(TokenSymbol::AUSD),
					DexShare::Erc20(erc20_address())
				)),
				H160::from_str("0x000000000000000000020000000001015dddfce5").ok()
			);

			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::encode_evm_address(CurrencyId::DexShare(
					DexShare::Erc20(erc20_address()),
					DexShare::Erc20(erc20_address())
				)),
				H160::from_str("0x00000000000000000002015dddfce5015dddfce5").ok()
			);

			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::encode_evm_address(CurrencyId::DexShare(
					DexShare::Token(TokenSymbol::ACA),
					DexShare::Erc20(erc20_address_not_exists())
				)),
				None
			);

			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::encode_evm_address(CurrencyId::DexShare(
					DexShare::Erc20(erc20_address()),
					DexShare::Erc20(erc20_address_not_exists())
				)),
				None
			);

			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::encode_evm_address(CurrencyId::DexShare(
					DexShare::LiquidCroadloan(1),
					DexShare::ForeignAsset(2)
				)),
				H160::from_str("0x0000000000000000000202000000010300000002").ok()
			);
			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::encode_evm_address(CurrencyId::DexShare(
					DexShare::ForeignAsset(2),
					DexShare::LiquidCroadloan(1)
				)),
				H160::from_str("0x0000000000000000000203000000020200000001").ok()
			);

			// StableAssetPoolToken
			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::encode_evm_address(CurrencyId::StableAssetPoolToken(1)),
				None
			);

			// LiquidCroadloan
			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::encode_evm_address(CurrencyId::LiquidCroadloan(1)),
				H160::from_str("0x0000000000000000000400000000000000000001").ok()
			);

			// ForeignAsset
			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::encode_evm_address(CurrencyId::ForeignAsset(1)),
				H160::from_str("0x0000000000000000000500000000000000000001").ok()
			);
		});
}

#[test]
fn decode_evm_address_works() {
	ExtBuilder::default()
		.balances(vec![(alice(), 1_000_000_000_000)])
		.build()
		.execute_with(|| {
			deploy_contracts();
			assert_ok!(with_transaction_result(|| -> DispatchResult {
				EvmErc20InfoMapping::<Runtime>::set_erc20_mapping(erc20_address())
			}));

			// Token
			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::decode_evm_address(
					EvmErc20InfoMapping::<Runtime>::encode_evm_address(CurrencyId::Token(TokenSymbol::ACA)).unwrap()
				),
				Some(CurrencyId::Token(TokenSymbol::ACA))
			);

			// Erc20
			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::decode_evm_address(
					EvmErc20InfoMapping::<Runtime>::encode_evm_address(CurrencyId::Erc20(erc20_address())).unwrap()
				),
				Some(CurrencyId::Erc20(erc20_address()))
			);

			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::decode_evm_address(
					EvmErc20InfoMapping::<Runtime>::encode_evm_address(CurrencyId::Erc20(erc20_address_not_exists()))
						.unwrap()
				),
				None,
			);

			// DexShare
			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::decode_evm_address(
					EvmErc20InfoMapping::<Runtime>::encode_evm_address(CurrencyId::DexShare(
						DexShare::Token(TokenSymbol::ACA),
						DexShare::Token(TokenSymbol::AUSD)
					))
					.unwrap(),
				),
				Some(CurrencyId::DexShare(
					DexShare::Token(TokenSymbol::ACA),
					DexShare::Token(TokenSymbol::AUSD)
				))
			);

			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::decode_evm_address(
					EvmErc20InfoMapping::<Runtime>::encode_evm_address(CurrencyId::DexShare(
						DexShare::Erc20(erc20_address()),
						DexShare::Token(TokenSymbol::AUSD)
					))
					.unwrap()
				),
				Some(CurrencyId::DexShare(
					DexShare::Erc20(erc20_address()),
					DexShare::Token(TokenSymbol::AUSD)
				))
			);

			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::decode_evm_address(
					EvmErc20InfoMapping::<Runtime>::encode_evm_address(CurrencyId::DexShare(
						DexShare::Erc20(erc20_address()),
						DexShare::Erc20(erc20_address())
					))
					.unwrap()
				),
				Some(CurrencyId::DexShare(
					DexShare::Erc20(erc20_address()),
					DexShare::Erc20(erc20_address())
				))
			);

			// decode invalid evm address
			// CurrencyId::DexShare(DexShare::Token(TokenSymbol::ACA),
			// DexShare::Erc20(erc20_address_not_exists()))
			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::decode_evm_address(
					H160::from_str("0x0000000000000000000000010000000002000001").unwrap()
				),
				None
			);

			// decode invalid evm address
			// CurrencyId::DexShare(DexShare::Erc20(erc20_address()),
			// DexShare::Erc20(erc20_address_not_exists()))
			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::decode_evm_address(
					H160::from_str("0x0000000000000000000000010200000002000001").unwrap()
				),
				None
			);

			// Allow non-system contracts
			let non_system_contracts = H160::from_str("0x1000000000000000000000000000000000000000").unwrap();
			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::decode_evm_address(non_system_contracts),
				Some(CurrencyId::Erc20(non_system_contracts))
			);

			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::decode_evm_address(
					EvmErc20InfoMapping::<Runtime>::encode_evm_address(CurrencyId::DexShare(
						DexShare::LiquidCroadloan(1),
						DexShare::ForeignAsset(2)
					))
					.unwrap()
				),
				Some(CurrencyId::DexShare(
					DexShare::LiquidCroadloan(1),
					DexShare::ForeignAsset(2)
				))
			);

			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::decode_evm_address(
					EvmErc20InfoMapping::<Runtime>::encode_evm_address(CurrencyId::DexShare(
						DexShare::ForeignAsset(2),
						DexShare::LiquidCroadloan(1),
					))
					.unwrap()
				),
				Some(CurrencyId::DexShare(
					DexShare::ForeignAsset(2),
					DexShare::LiquidCroadloan(1)
				))
			);

			// StableAssetPoolToken
			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::decode_evm_address(
					H160::from_str("0x0000000000000000000000030000000000000000").unwrap()
				),
				None
			);
			// LiquidCroadloan
			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::decode_evm_address(
					EvmErc20InfoMapping::<Runtime>::encode_evm_address(CurrencyId::LiquidCroadloan(1)).unwrap()
				),
				Some(CurrencyId::LiquidCroadloan(1))
			);

			// ForeignAsset
			assert_eq!(
				EvmErc20InfoMapping::<Runtime>::decode_evm_address(
					EvmErc20InfoMapping::<Runtime>::encode_evm_address(CurrencyId::ForeignAsset(1)).unwrap()
				),
				Some(CurrencyId::ForeignAsset(1))
			);
		});
}
