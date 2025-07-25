use blake2::Blake2b;
use digest::consts::U32;
use mithril_stm::{
    BasicVerifier, CoreVerifierError, Initializer, Parameters, Signer, SingleSignature, Stake,
    VerificationKey,
};
use rand_chacha::ChaCha20Rng;
use rand_core::{RngCore, SeedableRng};
type D = Blake2b<U32>;

#[test]
fn test_core_verifier() {
    let nparties: usize = 32;
    let mut rng = ChaCha20Rng::from_seed([0u8; 32]);
    let mut msg = [0u8; 32];
    rng.fill_bytes(&mut msg);
    let mut public_signers: Vec<(VerificationKey, Stake)> = Vec::with_capacity(nparties);
    let mut initializers: Vec<Initializer> = Vec::with_capacity(nparties);

    //////////////////////////
    // initialization phase //
    //////////////////////////

    let params = Parameters {
        k: 357,
        m: 2642,
        phi_f: 0.2,
    };

    let parties = (0..nparties).map(|_| 1 + (rng.next_u64() % 9999)).collect::<Vec<_>>();

    for stake in parties {
        let initializer = Initializer::new(params, stake, &mut rng);
        initializers.push(initializer.clone());
        public_signers.push((
            initializer.get_verification_key_proof_of_possession().vk,
            initializer.stake,
        ));
    }

    let core_verifier = BasicVerifier::new(&public_signers);

    let signers: Vec<Signer<D>> = initializers
        .into_iter()
        .filter_map(|s| s.create_basic_signer(&core_verifier.eligible_parties))
        .collect();

    //////////////////////////
    ///// operation phase ////
    //////////////////////////

    let mut signatures: Vec<SingleSignature> = Vec::with_capacity(nparties);
    for s in signers {
        if let Some(sig) = s.basic_sign(&msg, core_verifier.total_stake) {
            signatures.push(sig);
        }
    }
    let verify_result = core_verifier.verify(&signatures, &params, &msg);

    match verify_result {
        Ok(_) => {
            assert!(
                verify_result.is_ok(),
                "Verification failed: {verify_result:?}"
            );
        }
        Err(CoreVerifierError::NoQuorum(nr_indices, _k)) => {
            assert!((nr_indices) < params.k);
        }
        Err(CoreVerifierError::IndexNotUnique) => unreachable!(),
        _ => unreachable!(),
    }
}
