use alloc::vec::Vec;
use core::marker::PhantomData;

use ark_bls12_381::Fr;
use ark_ff::UniformRand;
use ark_serialize::{CanonicalSerialize, Write};
use ark_std::rand::rngs::StdRng;
use ark_std::rand::SeedableRng;
use sha2::Digest;

use crate::data_structures::KzgCommitment;

#[derive(Clone, Default)]
pub struct ChallengeGenerator<T: Digest + Default> {
    data: Option<Vec<u8>>,
    generated: bool,
    _phantom_data: PhantomData<T>,
}

impl<T: Digest + Default> ChallengeGenerator<T> {
    pub fn from_commitment(kzg_commitment: &[KzgCommitment]) -> Self {
        let mut challenge_parse = Self::default();
        for commitment in kzg_commitment {
            challenge_parse.feed(commitment);
        }
        challenge_parse
    }
}

impl<T: Digest + Default> ChallengeGenerator<T> {
    pub fn feed(&mut self, kzg_commitment: &KzgCommitment) {
        let mut hasher = T::default();
        hasher.update(self.data.take().unwrap_or_default());
        kzg_commitment
            .inner()
            .serialize_uncompressed(HashMarshaller(&mut hasher))
            .expect("HashMarshaller::serialize_uncompressed should be infallible!");
        self.data = Some(hasher.finalize().to_vec());
        self.generated = false;
    }

    fn generate_rng_with_seed(&mut self) -> StdRng {
        if self.generated {
            panic!("I'm hungry! Feed me something first");
        }
        self.generated = true;
        let mut seed: [u8; 8] = Default::default();
        seed.copy_from_slice(&self.data.clone().unwrap_or_default()[0..8]);
        let seed = u64::from_le_bytes(seed);
        StdRng::seed_from_u64(seed)
    }


    pub fn generate_challenges<const N: usize>(&mut self) -> [Fr; N] {
        let mut rng = self.generate_rng_with_seed();
        let points = [0; N];
        points.map(|_| Fr::rand(&mut rng))
    }
}

struct HashMarshaller<'a, H: Digest>(&'a mut H);

impl<'a, H: Digest> Write for HashMarshaller<'a, H> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> ark_std::io::Result<usize> {
        Digest::update(self.0, buf);
        Ok(buf.len())
    }
    #[inline]
    fn flush(&mut self) -> ark_std::io::Result<()> {
        Ok(())
    }
}
