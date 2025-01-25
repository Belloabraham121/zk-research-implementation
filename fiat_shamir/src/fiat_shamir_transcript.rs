use ark_bn254::Fq;
use ark_ff::{BigInteger, PrimeField};
use sha3::{Digest, Keccak256};
use std::marker::PhantomData;

#[derive(Clone)]
pub struct Transcript<F: PrimeField> {
    _field: PhantomData<F>,
    hasher: Keccak256,
}

impl<F: PrimeField> Transcript<F> {
    pub fn new() -> Self {
        Self {
            _field: PhantomData,
            hasher: Keccak256::new(),
        }
    }

    pub fn append(&mut self, preimage: &[u8]) {
        self.hasher.update(preimage)
    }

    pub fn get_random_challenge(&mut self) -> F {
        let random_challenge = self.hasher.finalize_reset();

        self.append(&random_challenge);

        F::from_le_bytes_mod_order(&random_challenge)
    }
}

pub fn fq_vec_to_bytes(values: &[Fq]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|x| x.into_bigint().to_bytes_le())
        .collect()
}

#[cfg(test)]
mod test {
    use super::Transcript;
    use ark_bn254::Fq;

    #[test]
    fn it_hashes() {
        let mut transcript: Transcript<Fq> = Transcript::new();
        transcript.append("zero knowledge".as_bytes());

        let random_challenge = transcript.get_random_challenge();

        dbg!(random_challenge);
    }
}
