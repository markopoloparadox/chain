
//! Autogenerated weights for `pallet_babe`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-03-27, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("alphanet-dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/ternoa
// benchmark
// --chain
// alphanet-dev
// --steps=50
// --repeat=20
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./runtime/alphanet/src/weights/
// --pallet=pallet_babe

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_babe`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_babe::WeightInfo for WeightInfo<T> {
	fn check_equivocation_proof(x: u32, ) -> Weight {
		(90_868_000 as Weight)
			// Standard Error: 47_000
			.saturating_add((392_000 as Weight).saturating_mul(x as Weight))
	}
}
