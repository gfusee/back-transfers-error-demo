#![no_std]

multiversx_sc::imports!();

#[multiversx_sc::contract]
pub trait MyContract: ContractBase {
    #[init]
    fn init(&self) {}

    #[endpoint(forwardPayment)]
    #[payable("*")]
    fn forward_payment(&self) {
        let esdt_payment = self.call_value().all_esdt_transfers();
        let caller = self.blockchain().get_caller();

        for payment in esdt_payment.into_iter() {
            self.send()
                .direct_esdt(
                    &caller,
                    &payment.token_identifier,
                    0,
                    &payment.amount
                )
        }
    }

    #[endpoint(callForward)]
    #[payable("*")]
    fn call_forward(&self, contract_address: ManagedAddress<Self::Api>) -> ManagedVec<EsdtTokenPayment<Self::Api>> {
        let esdt_payments = self.call_value().all_esdt_transfers();

        let (_, back_transfers) = if esdt_payments.is_empty() {
            self.self_proxy(contract_address)
                .forward_payment()
                .execute_on_dest_context_with_back_transfers::<IgnoreValue>()
        } else {
            self.self_proxy(contract_address)
                .forward_payment()
                .with_esdt_transfer(self.call_value().single_esdt())
                .execute_on_dest_context_with_back_transfers::<IgnoreValue>()
        };

        require!(
            back_transfers.total_egld_amount == 0,
            "egld payment received but expected esdt payment"
        );

        back_transfers.esdt_payments
    }

    #[proxy]
    fn self_proxy(&self, address: ManagedAddress<Self::Api>) -> crate::Proxy<Self::Api>;
}
