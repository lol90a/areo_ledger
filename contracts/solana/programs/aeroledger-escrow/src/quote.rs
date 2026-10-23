//! Signed price-quote attestations.
//!
//! The on-chain program must never let a buyer invent the amount they pay. The
//! authoritative price is computed off-chain by the AeroLedger backend, which
//! signs a canonical quote message with its `quote_authority` Ed25519 key. The
//! buyer embeds that signature in the SAME transaction as `create_order`, using
//! Solana's native Ed25519 signature-verification program, and this module
//! proves -- by introspecting the instruction list -- that the verification
//! actually happened, over the exact bytes this order commits to.
//!
//! Everything here is pure (`&[u8]` in, `Result` out) so it is unit-testable
//! with plain `cargo test`, without a validator.

use anchor_lang::prelude::*;

/// Domain-separation prefix. Changing the quote layout MUST change this string,
/// so a signature produced for an older layout can never be replayed against a
/// newer one.
pub const QUOTE_DOMAIN: &[u8] = b"AEROLEDGER_ORDER_QUOTE_V1";

/// Byte length of a canonical quote message.
/// domain(25) + order_id_hash(32) + buyer(32) + seller(32) + mint(32)
/// + amount(8) + fee_amount(8) + expiry(8)
pub const QUOTE_MESSAGE_LEN: usize = 25 + 32 + 32 + 32 + 32 + 8 + 8 + 8;

/// The price commitment the backend signs and the program enforces.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct QuoteCommitment {
    /// Total token base units the buyer must escrow.
    pub amount: u64,
    /// Platform fee, in the same base units, taken out of `amount` at payout.
    pub fee_amount: u64,
    /// Unix timestamp after which this quote is no longer valid on-chain.
    pub expiry: i64,
}

/// Build the canonical message for a quote. Field order and widths are fixed;
/// every integer is little-endian. Any change here is a breaking protocol change
/// and must be paired with a new [`QUOTE_DOMAIN`].
pub fn quote_message(
    order_id_hash: &[u8; 32],
    buyer: &Pubkey,
    seller: &Pubkey,
    mint: &Pubkey,
    quote: &QuoteCommitment,
) -> [u8; QUOTE_MESSAGE_LEN] {
    let mut out = [0u8; QUOTE_MESSAGE_LEN];
    let mut at = 0usize;

    fn put(src: &[u8], out: &mut [u8; QUOTE_MESSAGE_LEN], at: &mut usize) {
        out[*at..*at + src.len()].copy_from_slice(src);
        *at += src.len();
    }

    put(QUOTE_DOMAIN, &mut out, &mut at);
    put(order_id_hash, &mut out, &mut at);
    put(buyer.as_ref(), &mut out, &mut at);
    put(seller.as_ref(), &mut out, &mut at);
    put(mint.as_ref(), &mut out, &mut at);
    put(&quote.amount.to_le_bytes(), &mut out, &mut at);
    put(&quote.fee_amount.to_le_bytes(), &mut out, &mut at);
    put(&quote.expiry.to_le_bytes(), &mut out, &mut at);

    debug_assert_eq!(at, QUOTE_MESSAGE_LEN);
    out
}

// Layout of the native Ed25519 program instruction data:
//
//   offset  size  field
//        0     1  num_signatures
//        1     1  padding
//        2     2  signature_offset
//        4     2  signature_instruction_index
//        6     2  public_key_offset
//        8     2  public_key_instruction_index
//       10     2  message_data_offset
//       12     2  message_data_size
//       14     2  message_instruction_index
const ED25519_HEADER_LEN: usize = 16;
const ED25519_SIGNATURE_LEN: usize = 64;
const ED25519_PUBKEY_LEN: usize = 32;
/// Sentinel the Ed25519 program uses for "data lives in this same instruction".
const SELF_INSTRUCTION_INDEX: u16 = u16::MAX;

fn read_u16_le(data: &[u8], at: usize) -> Option<u16> {
    let bytes = data.get(at..at + 2)?;
    Some(u16::from_le_bytes([bytes[0], bytes[1]]))
}

/// Errors that can come out of parsing an Ed25519 verification instruction.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum QuoteSignatureError {
    /// The instruction data is too short or structurally malformed.
    Malformed,
    /// More or fewer than exactly one signature was verified.
    NotExactlyOneSignature,
    /// A field points at a different instruction -- we only accept self-contained
    /// Ed25519 instructions, so nothing can be smuggled in from elsewhere.
    IndirectData,
    /// The signature was made by a key other than the configured quote authority.
    WrongSigner,
    /// The signed message is not the message this order commits to.
    MessageMismatch,
}

/// Verify that `ed25519_ix_data` is a well-formed, self-contained Ed25519
/// verification of `expected_message` by `expected_signer`.
///
/// This does NOT verify the signature cryptographically -- the Ed25519 native
/// program already did that, and the runtime fails the whole transaction if it
/// did not. What this function proves is that the verification the runtime
/// performed was over OUR message and OUR signer, rather than over attacker-chosen
/// bytes. That distinction is the whole security argument: an attacker can put any
/// Ed25519 instruction they like in the transaction, so the program has to pin
/// down exactly which signature it is relying on.
pub fn verify_quote_signature_data(
    ed25519_ix_data: &[u8],
    expected_signer: &Pubkey,
    expected_message: &[u8],
) -> std::result::Result<(), QuoteSignatureError> {
    if ed25519_ix_data.len() < ED25519_HEADER_LEN {
        return Err(QuoteSignatureError::Malformed);
    }
    // Exactly one signature: a multi-signature instruction would let an attacker
    // bury our expected signature alongside others and reuse offsets ambiguously.
    if ed25519_ix_data[0] != 1 {
        return Err(QuoteSignatureError::NotExactlyOneSignature);
    }

    let signature_offset = read_u16_le(ed25519_ix_data, 2).ok_or(QuoteSignatureError::Malformed)?;
    let signature_ix_index =
        read_u16_le(ed25519_ix_data, 4).ok_or(QuoteSignatureError::Malformed)?;
    let pubkey_offset = read_u16_le(ed25519_ix_data, 6).ok_or(QuoteSignatureError::Malformed)?;
    let pubkey_ix_index = read_u16_le(ed25519_ix_data, 8).ok_or(QuoteSignatureError::Malformed)?;
    let message_offset = read_u16_le(ed25519_ix_data, 10).ok_or(QuoteSignatureError::Malformed)?;
    let message_size = read_u16_le(ed25519_ix_data, 12).ok_or(QuoteSignatureError::Malformed)?;
    let message_ix_index =
        read_u16_le(ed25519_ix_data, 14).ok_or(QuoteSignatureError::Malformed)?;

    // Every component must live inside THIS instruction. Otherwise the Ed25519
    // program would be verifying bytes held in some other instruction of the
    // transaction, which we have not inspected and must not trust.
    if signature_ix_index != SELF_INSTRUCTION_INDEX
        || pubkey_ix_index != SELF_INSTRUCTION_INDEX
        || message_ix_index != SELF_INSTRUCTION_INDEX
    {
        return Err(QuoteSignatureError::IndirectData);
    }

    let sig_start = signature_offset as usize;
    let key_start = pubkey_offset as usize;
    let msg_start = message_offset as usize;
    let msg_len = message_size as usize;

    // Bounds-check everything before slicing.
    let sig_end = sig_start
        .checked_add(ED25519_SIGNATURE_LEN)
        .ok_or(QuoteSignatureError::Malformed)?;
    let key_end = key_start
        .checked_add(ED25519_PUBKEY_LEN)
        .ok_or(QuoteSignatureError::Malformed)?;
    let msg_end = msg_start
        .checked_add(msg_len)
        .ok_or(QuoteSignatureError::Malformed)?;

    let _sig = ed25519_ix_data
        .get(sig_start..sig_end)
        .ok_or(QuoteSignatureError::Malformed)?;
    let key = ed25519_ix_data
        .get(key_start..key_end)
        .ok_or(QuoteSignatureError::Malformed)?;
    let msg = ed25519_ix_data
        .get(msg_start..msg_end)
        .ok_or(QuoteSignatureError::Malformed)?;

    if key != expected_signer.as_ref() {
        return Err(QuoteSignatureError::WrongSigner);
    }
    if msg != expected_message {
        return Err(QuoteSignatureError::MessageMismatch);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pk(seed: u8) -> Pubkey {
        Pubkey::new_from_array([seed; 32])
    }

    fn sample_quote() -> QuoteCommitment {
        QuoteCommitment {
            amount: 1_500_000,
            fee_amount: 45_000,
            expiry: 1_900_000_000,
        }
    }

    /// Build a valid, self-contained Ed25519 instruction payload for `message`
    /// signed (nominally) by `signer`. The 64 signature bytes are arbitrary here:
    /// this module deliberately does not re-check the cryptography, only the
    /// binding between the verified bytes and the order.
    fn ed25519_ix_data(signer: &Pubkey, message: &[u8]) -> Vec<u8> {
        let pubkey_offset = ED25519_HEADER_LEN;
        let signature_offset = pubkey_offset + ED25519_PUBKEY_LEN;
        let message_offset = signature_offset + ED25519_SIGNATURE_LEN;

        let mut data = Vec::with_capacity(message_offset + message.len());
        data.push(1); // num_signatures
        data.push(0); // padding
        data.extend_from_slice(&(signature_offset as u16).to_le_bytes());
        data.extend_from_slice(&SELF_INSTRUCTION_INDEX.to_le_bytes());
        data.extend_from_slice(&(pubkey_offset as u16).to_le_bytes());
        data.extend_from_slice(&SELF_INSTRUCTION_INDEX.to_le_bytes());
        data.extend_from_slice(&(message_offset as u16).to_le_bytes());
        data.extend_from_slice(&(message.len() as u16).to_le_bytes());
        data.extend_from_slice(&SELF_INSTRUCTION_INDEX.to_le_bytes());
        data.extend_from_slice(signer.as_ref());
        data.extend_from_slice(&[7u8; ED25519_SIGNATURE_LEN]);
        data.extend_from_slice(message);
        data
    }

    #[test]
    fn message_has_the_documented_length_and_layout() {
        let msg = quote_message(&[9u8; 32], &pk(1), &pk(2), &pk(3), &sample_quote());
        assert_eq!(msg.len(), QUOTE_MESSAGE_LEN);
        assert_eq!(&msg[..QUOTE_DOMAIN.len()], QUOTE_DOMAIN);
        // amount sits right after domain + 4 pubkey-sized fields.
        let amount_at = QUOTE_DOMAIN.len() + 32 * 4;
        assert_eq!(
            u64::from_le_bytes(msg[amount_at..amount_at + 8].try_into().unwrap()),
            1_500_000
        );
    }

    #[test]
    fn every_field_changes_the_message() {
        let base = quote_message(&[9u8; 32], &pk(1), &pk(2), &pk(3), &sample_quote());

        assert_ne!(
            base,
            quote_message(&[8u8; 32], &pk(1), &pk(2), &pk(3), &sample_quote())
        );
        assert_ne!(
            base,
            quote_message(&[9u8; 32], &pk(4), &pk(2), &pk(3), &sample_quote())
        );
        assert_ne!(
            base,
            quote_message(&[9u8; 32], &pk(1), &pk(4), &pk(3), &sample_quote())
        );
        assert_ne!(
            base,
            quote_message(&[9u8; 32], &pk(1), &pk(2), &pk(4), &sample_quote())
        );

        let mut cheaper = sample_quote();
        cheaper.amount = 1;
        assert_ne!(
            base,
            quote_message(&[9u8; 32], &pk(1), &pk(2), &pk(3), &cheaper)
        );

        let mut no_fee = sample_quote();
        no_fee.fee_amount = 0;
        assert_ne!(
            base,
            quote_message(&[9u8; 32], &pk(1), &pk(2), &pk(3), &no_fee)
        );

        let mut later = sample_quote();
        later.expiry += 1;
        assert_ne!(
            base,
            quote_message(&[9u8; 32], &pk(1), &pk(2), &pk(3), &later)
        );
    }

    #[test]
    fn accepts_a_well_formed_self_contained_verification() {
        let signer = pk(42);
        let msg = quote_message(&[9u8; 32], &pk(1), &pk(2), &pk(3), &sample_quote());
        let data = ed25519_ix_data(&signer, &msg);
        assert_eq!(verify_quote_signature_data(&data, &signer, &msg), Ok(()));
    }

    #[test]
    fn rejects_a_signature_from_the_wrong_authority() {
        let msg = quote_message(&[9u8; 32], &pk(1), &pk(2), &pk(3), &sample_quote());
        let data = ed25519_ix_data(&pk(66), &msg);
        assert_eq!(
            verify_quote_signature_data(&data, &pk(42), &msg),
            Err(QuoteSignatureError::WrongSigner)
        );
    }

    #[test]
    fn rejects_a_signature_over_a_cheaper_amount() {
        let signer = pk(42);
        let mut cheaper = sample_quote();
        cheaper.amount = 1;

        // The attacker holds a genuine backend signature, but over a different
        // (cheaper) quote than the one the order account will be built from.
        let signed = quote_message(&[9u8; 32], &pk(1), &pk(2), &pk(3), &cheaper);
        let data = ed25519_ix_data(&signer, &signed);

        let committed = quote_message(&[9u8; 32], &pk(1), &pk(2), &pk(3), &sample_quote());
        assert_eq!(
            verify_quote_signature_data(&data, &signer, &committed),
            Err(QuoteSignatureError::MessageMismatch)
        );
    }

    #[test]
    fn rejects_a_quote_issued_to_a_different_buyer() {
        let signer = pk(42);
        let victim_quote = quote_message(&[9u8; 32], &pk(1), &pk(2), &pk(3), &sample_quote());
        let data = ed25519_ix_data(&signer, &victim_quote);

        let attacker_order = quote_message(&[9u8; 32], &pk(99), &pk(2), &pk(3), &sample_quote());
        assert_eq!(
            verify_quote_signature_data(&data, &signer, &attacker_order),
            Err(QuoteSignatureError::MessageMismatch)
        );
    }

    #[test]
    fn rejects_multi_signature_instructions() {
        let signer = pk(42);
        let msg = quote_message(&[9u8; 32], &pk(1), &pk(2), &pk(3), &sample_quote());
        let mut data = ed25519_ix_data(&signer, &msg);
        data[0] = 2;
        assert_eq!(
            verify_quote_signature_data(&data, &signer, &msg),
            Err(QuoteSignatureError::NotExactlyOneSignature)
        );
    }

    #[test]
    fn rejects_zero_signature_instructions() {
        let signer = pk(42);
        let msg = quote_message(&[9u8; 32], &pk(1), &pk(2), &pk(3), &sample_quote());
        let mut data = ed25519_ix_data(&signer, &msg);
        data[0] = 0;
        assert_eq!(
            verify_quote_signature_data(&data, &signer, &msg),
            Err(QuoteSignatureError::NotExactlyOneSignature)
        );
    }

    #[test]
    fn rejects_data_borrowed_from_another_instruction() {
        let signer = pk(42);
        let msg = quote_message(&[9u8; 32], &pk(1), &pk(2), &pk(3), &sample_quote());
        let mut data = ed25519_ix_data(&signer, &msg);
        // Point the message at instruction 0 instead of "self".
        data[14..16].copy_from_slice(&0u16.to_le_bytes());
        assert_eq!(
            verify_quote_signature_data(&data, &signer, &msg),
            Err(QuoteSignatureError::IndirectData)
        );
    }

    #[test]
    fn rejects_truncated_and_out_of_bounds_payloads() {
        let signer = pk(42);
        let msg = quote_message(&[9u8; 32], &pk(1), &pk(2), &pk(3), &sample_quote());
        let data = ed25519_ix_data(&signer, &msg);

        assert_eq!(
            verify_quote_signature_data(&data[..10], &signer, &msg),
            Err(QuoteSignatureError::Malformed)
        );

        // A message offset past the end of the buffer must not panic.
        let mut oob = data.clone();
        oob[10..12].copy_from_slice(&60_000u16.to_le_bytes());
        assert_eq!(
            verify_quote_signature_data(&oob, &signer, &msg),
            Err(QuoteSignatureError::Malformed)
        );
    }

    #[test]
    fn rejects_a_message_that_is_a_prefix_of_the_expected_one() {
        let signer = pk(42);
        let msg = quote_message(&[9u8; 32], &pk(1), &pk(2), &pk(3), &sample_quote());
        let data = ed25519_ix_data(&signer, &msg[..QUOTE_MESSAGE_LEN - 8]);
        assert_eq!(
            verify_quote_signature_data(&data, &signer, &msg),
            Err(QuoteSignatureError::MessageMismatch)
        );
    }
}
