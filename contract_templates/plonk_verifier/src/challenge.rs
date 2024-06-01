use alloc::vec::Vec;
use core::marker::PhantomData;

use ark_bls12_381::Fr;
use ark_ff::UniformRand;
use ark_serialize::{CanonicalSerialize, Write};
use ark_std::rand::rngs::StdRng;
use ark_std::rand::SeedableRng;
use sha2::Digest;

use crate::data_structures::KzgCommitment;

/// A struct for generating challenges using a cryptographic hash function.
#[derive(Clone, Default)]
pub struct ChallengeGenerator<T: Digest + Default> {
    data: Option<Vec<u8>>,
    generated: bool,
    _phantom_data: PhantomData<T>,
}

impl<T: Digest + Default> ChallengeGenerator<T> {
    /// Creates a new `ChallengeGenerator` from a slice of KZG commitments.
    ///
    /// # Arguments
    ///
    /// * `kzg_commitments` - A slice of KZG commitments used to initialize the generator.
    ///
    /// # Returns
    ///
    /// A `ChallengeGenerator` initialized with the provided commitments.
    pub fn from_commitments(kzg_commitments: &[KzgCommitment]) -> Self {
        let mut challenge_generator = Self::default();
        for commitment in kzg_commitments {
            challenge_generator.feed(commitment);
        }
        challenge_generator
    }

    /// Feeds a KZG commitment into the generator to update its internal state.
    ///
    /// # Arguments
    ///
    /// * `kzg_commitment` - A reference to a KZG commitment to be fed into the generator.
    pub fn feed(&mut self, kzg_commitment: &KzgCommitment) {
        let mut hasher = T::default();
        if let Some(data) = self.data.take() {
            hasher.update(data);
        }
        kzg_commitment
            .inner()
            .serialize_uncompressed(&mut HashMarshaller(&mut hasher))
            .expect("Serialization should be infallible!");
        self.data = Some(hasher.finalize().to_vec());
        self.generated = false;
    }

    /// Generates a random number generator (RNG) seeded with the internal state of the generator.
    ///
    /// # Panics
    ///
    /// This function will panic if called without feeding the generator with new data.
    fn generate_rng_with_seed(&mut self) -> StdRng {
        if self.generated {
            panic!("Generator has already been used. Feed it new data before generating again.");
        }
        self.generated = true;
        let seed_bytes = self
            .data
            .as_ref()
            .expect("No data available to generate seed")[0..8]
            .try_into()
            .expect("Failed to generate seed from data");
        let seed = u64::from_le_bytes(seed_bytes);
        StdRng::seed_from_u64(seed)
    }

    /// Generates an array of random challenges.
    ///
    /// # Type Parameters
    ///
    /// * `N` - The number of challenges to generate.
    ///
    /// # Returns
    ///
    /// An array of `Fr` elements representing the generated challenges.
    pub fn generate_challenges<const N: usize>(&mut self) -> [Fr; N] {
        let mut rng = self.generate_rng_with_seed();
        let mut challenges = [Fr::default(); N];
        for challenge in &mut challenges {
            *challenge = Fr::rand(&mut rng);
        }
        challenges
    }
}

/// A struct for marshalling data into a hash digest.
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
