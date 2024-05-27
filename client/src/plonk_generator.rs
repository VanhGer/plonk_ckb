
use ark_bls12_381::{Bls12_381, Fr};
use ark_ff::{One, Zero};
use ark_poly::univariate::DensePolynomial;
use ark_poly_commit::marlin_pc::MarlinKZG10;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_std::test_rng;
use blake2::Blake2s;
use ckb_types::bytes::Bytes;
use plonk_ckb::{Composer, Plonk};

type PC = MarlinKZG10<Bls12_381, DensePolynomial<Fr>>;
type PlonkInst = Plonk<Fr, Blake2s, PC>;

fn ks() -> [Fr; 4] {
    [
        Fr::zero(),
        Fr::from(8_u64),
        Fr::from(13_u64),
        Fr::from(17_u64),
    ]
}

fn circuit() -> Composer<Fr>
{
    let mut cs = Composer::new();
    let one = Fr::one();
    let two = one + one;
    let three = two + one;
    let four = two + two;
    let six = two + four;
    let var_one = cs.alloc_and_assign(one);
    let var_two = cs.alloc_and_assign(two);
    let var_three = cs.alloc_and_assign(three);
    let var_four = cs.alloc_and_assign(four);
    let var_six = cs.alloc_and_assign(six);
    cs.create_add_gate(
        (var_one, one),
        (var_two, one),
        var_three,
        None,
        Fr::zero(),
        Fr::zero(),
    );
    cs.create_add_gate(
        (var_one, one),
        (var_three, one),
        var_four,
        None,
        Fr::zero(),
        Fr::zero(),
    );
    cs.create_mul_gate(
        var_two,
        var_two,
        var_four,
        None,
        Fr::one(),
        Fr::zero(),
        Fr::zero(),
    );
    cs.create_mul_gate(var_one, var_two, var_six, None, two, two, Fr::zero());
    cs.constrain_to_constant(var_six, six, Fr::zero());

    cs
}

pub fn generate_plonk() -> (Bytes, Bytes, Bytes){
    let rng = &mut test_rng();

    // compose
    let cs = circuit();
    let ks = ks();
    println!("Plonk: size of the circuit: {}", cs.size());

    println!("Plonk: setting up srs...");
    let srs = PlonkInst::setup(8, rng).unwrap();

    println!("Plonk: generating keys...");
    let (pk, vk) = PlonkInst::keygen(&srs, &cs, ks).unwrap();
    let mut vk_bytes = Vec::new();
    // println!("before serialize: {:?}", vk);
    vk.serialize_unchecked(&mut vk_bytes).unwrap();

    // println!("Plonk: VerifyKey length: {}", vk_bytes.len());

    // let new_vk = plonk_ckb::VerifierKey::<Fr, PC>::deserialize_unchecked(&vk_bytes[..]).unwrap();
    //assert_eq!(vk, new_vk);
    // println!("after serialize: {:?}", new_vk);
    println!("Plonk: proving...");
    let proof = PlonkInst::prove(&pk, &cs, rng).unwrap();
    let mut proof_bytes = Vec::new();
    proof.serialize_unchecked(&mut proof_bytes).unwrap();

    println!("Plonk: proof length: {}", proof_bytes.len());

    println!("{:?}", PlonkInst::verify(&vk, cs.public_inputs(), proof));

    let mut public_bytes = Vec::new();
    cs.public_inputs()
    .to_vec()
    .serialize_unchecked(&mut public_bytes)
    .unwrap();

    //let mut new_publics = Vec::new();
    let new_publics = Vec::<Fr>::deserialize_unchecked(&public_bytes[..]).unwrap();
    assert_eq!(cs.public_inputs(), new_publics);

    // println!("{:?}", PlonkInst::verify(&vk, &new_publics, new_proof));

    println!("Plonk: verifying on CKB...");

    (
        vk_bytes.into(),
        proof_bytes.into(),
        public_bytes.into(),
    )
}