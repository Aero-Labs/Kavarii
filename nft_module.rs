elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use elrond_wasm::elrond_codec::TopEncode;

const NFT_AMOUNT: u32 = 1;
const ROYALTIES_MAX: u32 = 10_000;

#[derive(TypeAbi, TopEncode, TopDecode)]
pub struct PriceTag<M: ManagedTypeApi> {
    pub token: TokenIdentifier<M>,
    pub nonce: u64,
    pub amount: BigUint<M>,
}

#[elrond_wasm::module]
pub trait NftModule {


    #[payable("*")]
    #[endpoint(issueToken)]
    fn issue_token(
        &self,
        #[payment] issue_cost: BigUint,
        token_name: ManagedBuffer,
        token_ticker: ManagedBuffer,
    ) {

        self.send()
                .esdt_system_sc_proxy()
                .issue_non_fungible(
                    issue_cost,
                    &token_name,
                    &token_ticker,
                    NonFungibleTokenProperties {
                        can_freeze: true,
                        can_wipe: true,
                        can_pause: true,
                        can_change_owner: true,
                        can_upgrade: true,
                        can_add_special_roles: true,
                    },
                )
                .async_call()
                .with_callback(self.callbacks().issue_callback())
                .call_and_exit()

    }

    #[endpoint(setLocalRoles)]
    fn set_local_roles(&self, nft_token_id: TokenIdentifier) {
        let caller = self.blockchain().get_caller();

        let collection_name = nft_token_id.clone();

        // save collection for playlist
        self.set_collection(&caller).push_back(collection_name);

        self.send()
            .esdt_system_sc_proxy()
            .set_special_roles(
                &self.blockchain().get_sc_address(),
                &nft_token_id,
                [EsdtLocalRole::NftCreate,EsdtLocalRole::NftBurn ][..].iter().cloned(),
            )
            .async_call()
            .call_and_exit()
    }


    #[view(getCollection)]
    #[storage_mapper("getCollection")]
    fn set_collection(&self, address: &ManagedAddress) -> LinkedListMapper<TokenIdentifier>;



    #[endpoint(transferOwner)]
    fn transfer_owner(&self, token_ident: TokenIdentifier) {
        let new_owner = self.blockchain().get_caller();
        self.send().esdt_system_sc_proxy().transfer_ownership(&token_ident, &new_owner.to_address()).async_call().call_and_exit();
    }














    // endpoints

    #[payable("*")]
    #[endpoint(buyNft)]
    fn buy_nft(&self, nft_nonce: u64) {
        let payment: EsdtTokenPayment<Self::Api> = self.call_value().payment();

        self.require_token_issued();
        require!(
            !self.price_tag(nft_nonce).is_empty(),
            "Invalid nonce or NFT was already sold"
        );

        let price_tag = self.price_tag(nft_nonce).get();
        require!(
            payment.token_identifier == price_tag.token,
            "Invalid token used as payment"
        );
        require!(
            payment.token_nonce == price_tag.nonce,
            "Invalid nonce for payment token"
        );
        require!(
            payment.amount == price_tag.amount,
            "Invalid amount as payment"
        );

        self.price_tag(nft_nonce).clear();

        let nft_token_id = self.nft_token_id().get();
        let caller = self.blockchain().get_caller();
        self.send().direct(
            &caller,
            &nft_token_id,
            nft_nonce,
            &BigUint::from(NFT_AMOUNT),
            &[],
        );

        let owner = self.blockchain().get_owner_address();
        self.send().direct(
            &owner,
            &payment.token_identifier,
            payment.token_nonce,
            &payment.amount,
            &[],
        );
    }

    // views

    #[allow(clippy::type_complexity)]
    #[view(getNftPrice)]
    fn get_nft_price(
        &self,
        nft_nonce: u64,
    ) -> OptionalValue<MultiValue3<TokenIdentifier, u64, BigUint>> {
        if self.price_tag(nft_nonce).is_empty() {
            // NFT was already sold
            OptionalValue::None
        } else {
            let price_tag = self.price_tag(nft_nonce).get();

            OptionalValue::Some((price_tag.token, price_tag.nonce, price_tag.amount).into())
        }
    }







    // callbacks

    #[callback]
    fn issue_callback(&self, #[call_result] result: ManagedAsyncCallResult<TokenIdentifier>) {
        match result {
            ManagedAsyncCallResult::Ok(token_id) => {
                self.nft_token_id().set(&token_id);
            },
            ManagedAsyncCallResult::Err(_) => {
                let caller = self.blockchain().get_caller();
                let (returned_tokens, token_id) = self.call_value().payment_token_pair();
                if token_id.is_egld() && returned_tokens > 0 {
                    self.send()
                        .direct(&caller, &token_id, 0, &returned_tokens, &[]);
                }
            },
        }
    }





















    // private

    #[allow(clippy::too_many_arguments)]
    fn create_nft_with_attributes<T: TopEncode>(
        &self,
        nft_token_id: TokenIdentifier,
        name: ManagedBuffer,
        royalties: BigUint,
        attributes: T,
        #[var_args] uri: MultiValueEncoded<ManagedBuffer>,
    ) -> u64 {
        self.require_token_issued();
        require!(royalties <= ROYALTIES_MAX, "Royalties cannot exceed 100%");

        let mut serialized_attributes = ManagedBuffer::new();
        if let core::result::Result::Err(err) = attributes.top_encode(&mut serialized_attributes) {
            sc_panic!("Attributes encode error: {}", err.message_bytes());
        }

        let attributes_sha256 = self
            .crypto()
            .sha256_legacy_managed::<1000>(&serialized_attributes);

        let attributes_hash = attributes_sha256.as_managed_buffer();


        let mut uris = ManagedVec::new();
        for single_uri in &uri.to_vec() {
            uris.push(single_uri);
        }



        let nft_nonce = self.send().esdt_nft_create(
           &nft_token_id,
            &BigUint::from(NFT_AMOUNT),
            &name,
            &royalties,
            attributes_hash,
            &attributes,
            &uris,
        );

        let caller = self.blockchain().get_caller();

        self.send().direct(
            &caller,
            &nft_token_id,
            nft_nonce,
            &BigUint::from(NFT_AMOUNT),
            &[],
        );

        nft_nonce
    }



    fn require_token_issued(&self) {
        require!(!self.nft_token_id().is_empty(), "Token not issued");
    }

    // storage

    #[storage_mapper("nftTokenId")]
    fn nft_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[storage_mapper("priceTag")]
    fn price_tag(&self, nft_nonce: u64) -> SingleValueMapper<PriceTag<Self::Api>>;
}
