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

use crate::{
	dollar, AccountId, AccumulatePeriod, CollateralCurrencyIds, Currencies, CurrencyId, GetNativeCurrencyId,
	GetStableCurrencyId, GetStakingCurrencyId, Incentives, Rate, Rewards, Runtime, System,
};

use super::utils::set_balance;
use frame_benchmarking::{account, whitelisted_caller, BenchmarkError};
use frame_support::traits::OnInitialize;
use frame_system::RawOrigin;
use module_incentives::PoolId;
use orml_benchmarking::runtime_benchmarks;
use orml_traits::MultiCurrency;
use primitives::DexShare;
use sp_std::prelude::*;

const SEED: u32 = 0;

const NATIVE: CurrencyId = GetNativeCurrencyId::get();
const STAKING: CurrencyId = GetStakingCurrencyId::get();
const STABLECOIN: CurrencyId = GetStableCurrencyId::get();

runtime_benchmarks! {
	{ Runtime, module_incentives }

	on_initialize {
		let c in 0 .. CollateralCurrencyIds::get().len().saturating_sub(1) as u32;
		let currency_ids = CollateralCurrencyIds::get();
		let block_number = AccumulatePeriod::get();

		for i in 0 .. c {
			let currency_id = currency_ids[i as usize];
			let pool_id = PoolId::Loans(currency_id);

			Incentives::update_incentive_rewards(RawOrigin::Root.into(), vec![(pool_id.clone(), vec![(NATIVE, 100 * dollar(NATIVE))])])?;
			orml_rewards::PoolInfos::<Runtime>::mutate(pool_id, |pool_info| {
				pool_info.total_shares += 100;
			});
		}

		Incentives::on_initialize(1);
		System::set_block_number(block_number);
	}: {
		Incentives::on_initialize(System::block_number());
	}

	deposit_dex_share {
		let caller: AccountId = whitelisted_caller();
		let native_stablecoin_lp = CurrencyId::join_dex_share_currency_id(NATIVE, STABLECOIN).unwrap();
		set_balance(native_stablecoin_lp, &caller, 10_000 * dollar(STABLECOIN));
	}: _(RawOrigin::Signed(caller), native_stablecoin_lp, 10_000 * dollar(STABLECOIN))

	withdraw_dex_share {
		let caller: AccountId = whitelisted_caller();
		let native_stablecoin_lp = CurrencyId::join_dex_share_currency_id(NATIVE, STABLECOIN).unwrap();
		set_balance(native_stablecoin_lp, &caller, 10_000 * dollar(STABLECOIN));
		Incentives::deposit_dex_share(
			RawOrigin::Signed(caller.clone()).into(),
			native_stablecoin_lp,
			10_000 * dollar(STABLECOIN)
		)?;
	}: _(RawOrigin::Signed(caller), native_stablecoin_lp, 8000 * dollar(STABLECOIN))

	claim_rewards {
		let caller: AccountId = whitelisted_caller();
		let pool_id = PoolId::Loans(STAKING);
		let native_currency_id = GetNativeCurrencyId::get();

		Rewards::add_share(&caller, &pool_id, 100);
		Currencies::deposit(native_currency_id, &Incentives::account_id(), 80 * dollar(native_currency_id))?;
		Rewards::accumulate_reward(&pool_id, native_currency_id, 80 * dollar(native_currency_id))?;
	}: _(RawOrigin::Signed(caller), pool_id)

	update_incentive_rewards {
		let c in 0 .. CollateralCurrencyIds::get().len().saturating_sub(1) as u32;
		let currency_ids = CollateralCurrencyIds::get();
		let mut updates = vec![];

		for i in 0 .. c {
			let currency_id = currency_ids[i as usize];
			updates.push((PoolId::Loans(currency_id), vec![(NATIVE, dollar(NATIVE))]));
		}
	}: _(RawOrigin::Root, updates)

	update_dex_saving_rewards {
		let c in 0 .. CollateralCurrencyIds::get().len().saturating_sub(1) as u32;
		let currency_ids = CollateralCurrencyIds::get();
		let caller: AccountId = account("caller", 0, SEED);
		let mut updates = vec![];
		let base_currency_id = GetStableCurrencyId::get();

		for i in 0 .. c {
			let currency_id = currency_ids[i as usize];
			let lp_share_currency_id = match (currency_id, base_currency_id) {
				(CurrencyId::Token(other_currency_symbol), CurrencyId::Token(base_currency_symbol)) => {
					CurrencyId::DexShare(DexShare::Token(other_currency_symbol), DexShare::Token(base_currency_symbol))
				}
				_ => return Err(BenchmarkError::Stop("invalid currency id")),
			};
			updates.push((PoolId::Dex(lp_share_currency_id), Rate::default()));
		}
	}: _(RawOrigin::Root, updates)

	update_claim_reward_deduction_rates {
		let c in 0 .. CollateralCurrencyIds::get().len().saturating_sub(1) as u32;
		let currency_ids = CollateralCurrencyIds::get();
		let mut updates = vec![];

		for i in 0 .. c {
			let currency_id = currency_ids[i as usize];
			updates.push((PoolId::Loans(currency_id), Rate::default()));
		}
	}: _(RawOrigin::Root, updates)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::benchmarking::utils::tests::new_test_ext;
	use orml_benchmarking::impl_benchmark_test_suite;

	impl_benchmark_test_suite!(new_test_ext(),);
}
