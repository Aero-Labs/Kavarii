#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

mod nft_module;
pub mod storage;
pub mod functions;


#[derive(TypeAbi, TopEncode, TopDecode)]
pub struct ExampleAttributes {
    pub creation_timestamp: u64,
}

#[elrond_wasm::contract]
pub trait NftMinter: nft_module::NftModule + storage::Storage + functions::Functions {
    #[init]
    fn init(&self,token_name: TokenIdentifier) {
        require!(token_name.is_egld() || token_name.is_valid_esdt_identifier(),"Invalid token provided");
        let contract_owner = &self.blockchain().get_caller();
        self.cf_token_name().set(&token_name);
        self.contract_owner().set(contract_owner);
    }

    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::redundant_closure)]

    #[endpoint(createNft)]
    fn create_nft(
        &self,
        nft_token_id: TokenIdentifier,
        name: ManagedBuffer,
        royalties: BigUint,
        attributes: ManagedBuffer,
        #[var_args] uri: MultiValueEncoded<ManagedBuffer>,
    ) {

        self.create_nft_with_attributes(
            nft_token_id,
            name,
            royalties,
            attributes,
            uri,
        );
    }


}

