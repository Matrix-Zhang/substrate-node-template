use crate as pallet_kitties;

use frame_support::{
	parameter_types,
	traits::{OnInitialize},
};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, Header as _, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system,
		RandomnessCollectiveFlip: pallet_randomness_collective_flip,
		Balances: pallet_balances,
		Kitties: pallet_kitties,
	}
);

parameter_types! {
	pub BlockWeights: frame_system::limits::BlockWeights =
		frame_system::limits::BlockWeights::simple_max(1024);

	pub const BlockHashCount: u64 = 250;
}
impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Call = Call;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
}

parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
	pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Config for Test {
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type Balance = u64;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}

impl pallet_randomness_collective_flip::Config for Test {}

parameter_types! {
	pub const ReservationFee: u64 = 5;
	pub const MaxKittyOwned: u32 = 3;
}

impl pallet_kitties::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type ReservableCurrency = Balances;
	type KittyRandomness = RandomnessCollectiveFlip;

	type MaxKittyOwned = MaxKittyOwned;
	type ReservationFee = ReservationFee;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	pallet_balances::GenesisConfig::<Test> { balances: vec![(1, 20), (2, 10), (3, 2)] }
		.assimilate_storage(&mut t)
		.unwrap();

	t.into()
}

pub fn setup_blocks(blocks: u64) {
	let mut parent_hash = System::parent_hash();

	for i in 1..(blocks + 1) {
		System::initialize(&i, &parent_hash, &Default::default(), frame_system::InitKind::Full);
		RandomnessCollectiveFlip::on_initialize(i);

		let header = System::finalize();
		parent_hash = header.hash();
		System::set_block_number(*header.number());
	}
}
