#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// A module for proof of existence
#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use sp_std::prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		#[pallet::constant]
		type MaxSize: Get<u32>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub (super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn proofs)]
	pub type Proofs<T: Config> =
		StorageMap<_, Blake2_128Concat, Vec<u8>, (T::AccountId, T::BlockNumber)>;

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		ClaimCreated(T::AccountId, Vec<u8>),
		ClaimRevoked(T::AccountId, Vec<u8>),
		ClaimTransferred(T::AccountId, T::AccountId, Vec<u8>),
	}

	#[pallet::error]
	pub enum Error<T> {
		NotClaimOwner,
		ProofAlreadyExist,
		ProofNotExist,
		ProofTooLong,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn create_claim(origin: OriginFor<T>, claim: Vec<u8>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(claim.len() <= T::MaxSize::get() as usize, <Error<T>>::ProofTooLong);
			ensure!(!(<Proofs<T>>::contains_key(&claim)), <Error<T>>::ProofAlreadyExist);
			<Proofs<T>>::insert(&claim, (who.clone(), <frame_system::Pallet<T>>::block_number()));
			Self::deposit_event(Event::ClaimCreated(who, claim));
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn revoke_claim(origin: OriginFor<T>, claim: Vec<u8>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let (owner, _) = <Proofs<T>>::get(&claim).ok_or_else(|| <Error<T>>::ProofNotExist)?;
			ensure!(owner == who, <Error<T>>::NotClaimOwner);
			<Proofs<T>>::remove(&claim);
			Self::deposit_event(Event::ClaimCreated(who, claim));
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn transfer_claim(
			origin: OriginFor<T>,
			to: T::AccountId,
			claim: Vec<u8>,
		) -> DispatchResult {
			let from = ensure_signed(origin)?;
			let (owner, block_number) =
				<Proofs<T>>::get(&claim).ok_or_else(|| <Error<T>>::ProofNotExist)?;
			ensure!(owner == from, <Error<T>>::NotClaimOwner);
			<Proofs<T>>::insert(&claim, (to.clone(), block_number));
			Self::deposit_event(Event::ClaimTransferred(from, to, claim));
			Ok(())
		}
	}
}
