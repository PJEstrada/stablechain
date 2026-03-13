use alloy::primitives::{Address, Bytes, U256};
use alloy::sol;
use alloy::sol_types::SolCall;

sol! {
    function balanceOf(address owner) external view returns (uint256 balance);
    function transfer(address to, uint256 amount) external returns (bool);
}

/// ABI-encoded calldata for `balanceOf(owner)`.
pub fn balance_of_calldata(owner: Address) -> Bytes {
    balanceOfCall { owner }.abi_encode().into()
}

/// Decode the raw bytes returned by an `eth_call` to `balanceOf`.
/// ABI-encoded uint256 is always 32 bytes, big-endian.
pub fn decode_u256_return(raw: &[u8]) -> U256 {
    if raw.len() < 32 {
        return U256::ZERO;
    }
    U256::from_be_slice(&raw[..32])
}

/// ABI-encoded calldata for `transfer(to, amount)`.
pub fn transfer_calldata(to: Address, amount: U256) -> Bytes {
    transferCall { to, amount }.abi_encode().into()
}
