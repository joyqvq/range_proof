
#[macro_use]
extern crate criterion;

mod range_proof_benches {

    use ark_bls12_381::Fr;
    use criterion::Criterion;
    use merlin::Transcript;
    use range_proof::{commitment_scheme, range_proof::RangeProof};

    fn proof(c: &mut Criterion) {
        let n = 32usize;
        // let n = 64usize;

        let (pk, _vk) = commitment_scheme::trusted_setup(4usize * n).unwrap();
        let z = Fr::from(100u8);
        let mut proof_transcript = Transcript::new(b"range_proof");
        c.bench_function("Proving", move |b| {
            b.iter(|| RangeProof::prove(&pk, n, &z, &mut proof_transcript))
        });
    }

    fn verify(c: &mut Criterion) {
        let n = 32usize;
        // let n = 64usize;
        let (pk, vk) = commitment_scheme::trusted_setup(4usize * n).unwrap();
        let z = Fr::from(100u8);
        let mut verification_transcript = Transcript::new(b"range_proof");
        let mut proof_transcript = Transcript::new(b"range_proof");
        let proof = RangeProof::prove(&pk, n, &z, &mut proof_transcript);

        c.bench_function("Verifying", move |b| {
            b.iter(|| RangeProof::verify(&proof, &vk, n, &mut verification_transcript))
        });
    }

    criterion_group! {
        name = range_proof_benches;
        config = Criterion::default().sample_size(100);
        targets = proof, verify,
    }
}

criterion_main!(range_proof_benches::range_proof_benches,);
