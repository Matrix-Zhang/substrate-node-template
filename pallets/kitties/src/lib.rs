#![cfg_attr(not(feature = "std"), no_std)]
#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		dispatch::DispatchResult,
		pallet_prelude::*,
		sp_runtime::traits::Hash,
		traits::{Currency, ExistenceRequirement, Randomness, ReservableCurrency},
		transactional,
	};
	use frame_system::pallet_prelude::*;
	use scale_info::TypeInfo;
	use sp_io::hashing::blake2_128;

	#[cfg(feature = "std")]
	use frame_support::serde::{Deserialize, Serialize};

	type AccountOf<T> = <T as frame_system::Config>::AccountId;
	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
	type DepositOf<T> = <<T as Config>::ReservableCurrency as Currency<
		<T as frame_system::Config>::AccountId,
	>>::Balance;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The Currency handler for the kitties pallet.
		type Currency: Currency<Self::AccountId>;

		type ReservableCurrency: ReservableCurrency<Self::AccountId>;

		type KittyRandomness: Randomness<Self::Hash, Self::BlockNumber>;

		#[pallet::constant]
		type MaxKittyOwned: Get<u32>;

		// The amount required to reserve a kitty.
		#[pallet::constant]
		type ReservationFee: Get<DepositOf<Self>>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub (super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub kitties: Vec<(T::AccountId, [u8; 16], Gender)>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> GenesisConfig<T> {
			GenesisConfig { kitties: vec![] }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			for (acct, dna, gender) in &self.kitties {
				let _ = <Pallet<T>>::mint(acct, Some(dna.clone()), Some(gender.clone()));
			}
		}
	}

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	#[cfg_attr(feature = "std", derive(Deserialize, Serialize))]
	pub enum Gender {
		Male,
		Female,
	}

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Kitty<T: Config> {
		pub dna: [u8; 16],
		pub price: Option<BalanceOf<T>>,
		pub gender: Gender,
		pub owner: AccountOf<T>,
	}

	#[pallet::storage]
	#[pallet::getter(fn kitty_cnt)]
	pub(super) type KittyCnt<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub(super) type KittiesOf<T: Config> = StorageMap<_, Twox64Concat, T::Hash, Kitty<T>>;

	#[pallet::storage]
	#[pallet::getter(fn kittes_owned)]
	pub(super) type KittiesOwned<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::AccountId,
		BoundedVec<T::Hash, T::MaxKittyOwned>,
		ValueQuery,
	>;

	#[pallet::error]
	pub enum Error<T> {
		BuyerIsKittyOwner,
		ExceedMaxKittyOwned,
		NotEnoughBalance,
		NotKittyOwner,
		KittyBidPriceTooLow,
		KittyCntOverflow,
		KittyNotExist,
		KittyNotForSale,
		TransferToSelf,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new Kitty was successfully created. \[sender, kitty_id, deposit\]
		Created(T::AccountId, T::Hash, DepositOf<T>),
		/// Kitty price was successfully set. \[sender, kitty_id, new_price\]
		PriceSet(T::AccountId, T::Hash, Option<BalanceOf<T>>),
		/// A Kitty was successfully transferred. \[from, to, kitty_id\]
		Transferred(T::AccountId, T::AccountId, T::Hash),
		/// A kitty was successfully bought. \[buyer, seller, kitty_id, bid_price\]
		Bought(T::AccountId, T::AccountId, T::Hash, BalanceOf<T>),
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[transactional]
		#[pallet::weight(100)]
		pub fn create_kitty(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let kitty_id = Self::mint(&sender, None, None)?;
			log::info!("A kitty is born with ID: {:?}", kitty_id);
			Ok(())
		}

		#[pallet::weight(100)]
		pub fn set_price(
			origin: OriginFor<T>,
			kitty_id: T::Hash,
			new_price: Option<BalanceOf<T>>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let mut kitty = Self::kitties(&kitty_id).ok_or_else(|| <Error<T>>::KittyNotExist)?;
			kitty.price = new_price;
			<KittiesOf<T>>::insert(kitty_id, kitty);
			Self::deposit_event(Event::PriceSet(sender, kitty_id, new_price));
			Ok(())
		}

		#[pallet::weight(100)]
		pub fn transfer(
			origin: OriginFor<T>,
			to: T::AccountId,
			kitty_id: T::Hash,
		) -> DispatchResult {
			let from = ensure_signed(origin)?;
			ensure!(Self::is_kitty_owner(&kitty_id, &from)?, <Error<T>>::NotKittyOwner);
			ensure!(from != to, <Error<T>>::TransferToSelf);

			let to_owned = <KittiesOwned<T>>::get(&to);
			ensure!(
				(to_owned.len() as u32) < T::MaxKittyOwned::get(),
				<Error<T>>::ExceedMaxKittyOwned
			);

			Self::transfer_kitty_to(&from, &to, &kitty_id)?;
			Self::deposit_event(Event::Transferred(from, to, kitty_id));
			Ok(())
		}

		#[transactional]
		#[pallet::weight(100)]
		pub fn buy_kitty(
			origin: OriginFor<T>,
			kitty_id: T::Hash,
			bid_price: BalanceOf<T>,
		) -> DispatchResult {
			let buyer = ensure_signed(origin)?;

			let kitty = Self::kitties(&kitty_id).ok_or(<Error<T>>::KittyNotExist)?;
			ensure!(kitty.owner != buyer, <Error<T>>::BuyerIsKittyOwner);

			if let Some(ask_price) = kitty.price {
				ensure!(ask_price <= bid_price, <Error<T>>::KittyBidPriceTooLow);
			} else {
				return Err(<Error<T>>::KittyNotForSale.into());
			}

			// Check the buyer has enough free balance
			ensure!(T::Currency::free_balance(&buyer) >= bid_price, <Error<T>>::NotEnoughBalance);

			// Make sure the buyer has the capacity to receive one more kitty
			let buyer_owned = <KittiesOwned<T>>::get(&buyer);
			ensure!(
				(buyer_owned.len() as u32) < T::MaxKittyOwned::get(),
				<Error<T>>::KittyCntOverflow
			);

			let seller = kitty.owner.clone();

			T::Currency::transfer(&buyer, &seller, bid_price, ExistenceRequirement::KeepAlive)?;
			Self::transfer_kitty_to(&seller, &buyer, &kitty_id)?;
			Self::deposit_event(Event::Bought(buyer, seller, kitty_id, bid_price));
			Ok(())
		}

		#[pallet::weight(100)]
		pub fn breed_kitty(
			origin: OriginFor<T>,
			parent1: T::Hash,
			parent2: T::Hash,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// Check: Verify `sender` owns both kitties (and both kitties exist).
			ensure!(Self::is_kitty_owner(&parent1, &sender)?, <Error<T>>::NotKittyOwner);
			ensure!(Self::is_kitty_owner(&parent2, &sender)?, <Error<T>>::NotKittyOwner);

			let new_dna = Self::breed_dna(&parent1, &parent2)?;
			let kitty_id = Self::mint(&sender, Some(new_dna), None)?;
			log::info!("A kitty is breed with ID: {:?}", kitty_id);
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn gen_gender() -> Gender {
			let random = T::KittyRandomness::random(&b"gender"[..]).0;
			match random.as_ref()[0] % 2 {
				0 => Gender::Male,
				_ => Gender::Female,
			}
		}

		fn gen_dna() -> [u8; 16] {
			let payload = (
				T::KittyRandomness::random(&b"dna"[..]).0,
				<frame_system::Pallet<T>>::block_number(),
			);
			payload.using_encoded(blake2_128)
		}

		fn is_kitty_owner(kitty_id: &T::Hash, acct: &T::AccountId) -> Result<bool, Error<T>> {
			match Self::kitties(kitty_id) {
				Some(kitty) => Ok(kitty.owner == *acct),
				None => Err(<Error<T>>::KittyNotExist),
			}
		}

		fn mint(
			owner: &T::AccountId,
			dna: Option<[u8; 16]>,
			gender: Option<Gender>,
		) -> Result<T::Hash, Error<T>> {
			let deposit = T::ReservationFee::get();
			T::ReservableCurrency::reserve(&owner, deposit)
				.map_err(|_| <Error<T>>::NotEnoughBalance)?;

			let kitty = Kitty {
				dna: dna.unwrap_or_else(Self::gen_dna),
				price: None,
				gender: gender.unwrap_or_else(Self::gen_gender),
				owner: owner.clone(),
			};

			let kitty_id = T::Hashing::hash_of(&kitty);
			let new_cnt = Self::kitty_cnt().checked_add(1).ok_or(<Error<T>>::KittyCntOverflow)?;

			<KittiesOwned<T>>::try_mutate(&owner, |kitty_vec| kitty_vec.try_push(kitty_id))
				.map_err(|_| <Error<T>>::ExceedMaxKittyOwned)?;

			<KittiesOf<T>>::insert(kitty_id, kitty);
			<KittyCnt<T>>::put(new_cnt);

			Self::deposit_event(Event::Created(owner.clone(), kitty_id, deposit));
			Ok(kitty_id)
		}

		#[transactional]
		fn transfer_kitty_to(
			from: &T::AccountId,
			to: &T::AccountId,
			kitty_id: &T::Hash,
		) -> Result<(), Error<T>> {
			let mut kitty = Self::kitties(&kitty_id).ok_or(<Error<T>>::KittyNotExist)?;
			let prev_owner = kitty.owner.clone();

			// Remove 'kitty_id' from the KittiesOwned vec of `prev_kitty_owner`
			<KittiesOwned<T>>::try_mutate(&prev_owner, |owned| {
				if let Some(index) = owned.iter().position(|&id| id == *kitty_id) {
					owned.swap_remove(index);
					return Ok(());
				}
				Err(())
			})
			.map_err(|_| <Error<T>>::KittyNotExist)?;

			kitty.owner = to.clone();
			// Reset the price, so the kitty is not for sale until `set_price` is called by new owner
			kitty.price = None;

			// Append the kitty to new owners owned vec
			<KittiesOwned<T>>::try_mutate(&to, |owned| owned.try_push(*kitty_id))
				.map_err(|_| <Error<T>>::KittyNotExist)?;
			<KittiesOf<T>>::insert(kitty_id, kitty);

			let deposit = T::ReservationFee::get();
			T::ReservableCurrency::unreserve(&from, deposit);
			Ok(())
		}

		pub fn breed_dna(parent1: &T::Hash, parent2: &T::Hash) -> Result<[u8; 16], Error<T>> {
			let dna1 = Self::kitties(parent1).ok_or(<Error<T>>::KittyNotExist)?.dna;
			let dna2 = Self::kitties(parent2).ok_or(<Error<T>>::KittyNotExist)?.dna;

			let mut new_dna = Self::gen_dna();
			for i in 0..new_dna.len() {
				new_dna[i] = (new_dna[i] & dna1[i]) | (!new_dna[i] & dna2[i]);
			}
			Ok(new_dna)
		}
	}
}
