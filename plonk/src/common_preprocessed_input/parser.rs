use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::ops::Neg;

use ark_bls12_381::Fr;
use ark_poly::univariate::DensePolynomial;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};

use crate::common_preprocessed_input::circuit::Circuit;
use crate::common_preprocessed_input::parser::TypeOfCircuit::{Addition, Multiplication};
use crate::constraint::{CopyConstraints, GateConstraints};

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
/// Enum defining circuit type
enum TypeOfCircuit {
    Addition,
    Multiplication,
    _Constant,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
struct ParserGate {
    //Left branch of the circuit
    pub left: ParserWire,
    //Right branch of the circuit
    pub right: ParserWire,
    //Bottom part (result) of the circuit
    pub bottom: ParserWire,
    //Type of circuit (addition/multiplication/constant)
    pub type_of_circuit: TypeOfCircuit,
}

impl ParserGate {
    fn new(left: ParserWire, right: ParserWire, bottom: ParserWire, type_of_circuit: TypeOfCircuit) -> Self {
        ParserGate {
            left,
            right,
            bottom,
            type_of_circuit,
        }
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
struct ParserWire {
    value_string: String,
    // value_fr: Fr,
}

impl ParserWire {
    fn new(value_string: String) -> Self {
        ParserWire {
            value_string,
            // value_fr,
        }
    }
}

#[derive(CanonicalSerialize, CanonicalDeserialize)]
struct CommonPreprocessedInput {
    n: usize,
    k1: Fr,
    k2: Fr,
    q_lx: DensePolynomial<Fr>,
    q_rx: DensePolynomial<Fr>,
    q_mx: DensePolynomial<Fr>,
    q_ox: DensePolynomial<Fr>,
    q_cx: DensePolynomial<Fr>,
    s_sigma_1: DensePolynomial<Fr>,
    s_sigma_2: DensePolynomial<Fr>,
    s_sigma_3: DensePolynomial<Fr>,
    pi_x: DensePolynomial<Fr>,
}

impl CommonPreprocessedInput {
    pub fn new(compiled_circuit: (GateConstraints, CopyConstraints, usize)) -> Self {
        let copy_constraint = compiled_circuit.1;
        let gate_constraint = compiled_circuit.0;
        Self {
            n: compiled_circuit.2,
            k1: copy_constraint.k1().clone(),
            k2: copy_constraint.k2().clone(),
            q_lx: gate_constraint.q_lx().clone(),
            q_rx: gate_constraint.q_rx().clone(),
            q_mx: gate_constraint.q_mx().clone(),
            q_ox: gate_constraint.q_ox().clone(),
            q_cx: gate_constraint.q_cx().clone(),
            s_sigma_1: copy_constraint.get_s_sigma_1().clone(),
            s_sigma_2: copy_constraint.get_s_sigma_2().clone(),
            s_sigma_3: copy_constraint.get_s_sigma_3().clone(),
            pi_x: gate_constraint.pi_x().clone(),
        }
    }
}

/// String to common processed input parser
///
/// See parse function for usage
#[derive(Default)]
pub struct Parser {}

impl Parser {

    /// Parse string into circuit
    ///
    /// ```
    /// ```
    pub fn parse(self, input: &str) {
        let input = Self::parse_string(input);
        let input = &input;

        //Step 1: prepare gate_list and position_map prior to coordinate pair accumulator
        let (gate_list, position_map) = self.prepare_generation(input);

        //Step 2: generate the actual circuit with coordinate pair accumulator for copy constraint
        let circuit = Self::gen_circuit(gate_list, position_map);

        let common_preprocessed_input = CommonPreprocessedInput::new(circuit.compile().unwrap());

        let mut bytes = Vec::new();

        common_preprocessed_input.serialize_uncompressed(&mut bytes).unwrap();
        let str = format!(
            "pub const COMMON_PROCESSED_INPUT:[u8;{}] = {:?};",
            bytes.len(),
            &bytes
        );
        fs::write("src/common_processed_input_const.rs", str).expect("write failed");
    }

    /// Generate [gate_list] and [position_map] to prepare for coordinate pair accumulator
    fn prepare_generation(
        &self,
        string: &str,
    ) -> (Vec<ParserGate>, HashMap<String, Vec<(usize, usize)>>) {
        let gate_list: RefCell<Vec<ParserGate>> = RefCell::new(Vec::new());
        let gate_set: RefCell<HashSet<ParserGate>> = RefCell::new(HashSet::new());
        //Map of integer key will be here, it will then be inserted into gen circuit method
        let position_map: RefCell<HashMap<String, Vec<(usize, usize)>>> =
            RefCell::new(HashMap::new());

        let result = string
            .split('=')
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        assert_eq!(result.len(), 2);
        let string = result[0].clone();
        let result = result[1].clone();

        let mut split_list: Vec<String> = string.split('+').map(|s| s.to_string()).collect();
        split_list.push("-".to_string() + result.as_str());
        split_list
            .into_iter()
            .map(|split_list| {
                split_list
                    .split('*')
                    .map(|s| s.trim().to_string())
                    .map(|s| ParserWire::new(s.clone()))
                    .collect::<Vec<ParserWire>>()
            })
            .map(|multi_collections| {
                let mut gate_list = gate_list.borrow_mut();
                let mut gate_set = gate_set.borrow_mut();
                let mut position_map = position_map.borrow_mut();
                multi_collections
                    .into_iter()
                    .reduce(|left, right| {
                        let gate_number = gate_list.len();
                        let result = ParserWire::new(
                            format!("{}*{}", &left.value_string, &right.value_string),
                            // left.value_fr * right.value_fr,
                        );
                        let gate =
                            ParserGate::new(left.clone(), right.clone(), result.clone(), Multiplication);
                        if gate_set.get(&gate).is_some() {
                            return result;
                        }
                        gate_list.push(gate.clone());
                        gate_set.insert(gate);

                        Self::push_into_position_map_or_insert(
                            0,
                            gate_number,
                            &mut position_map,
                            &left.value_string,
                        );
                        Self::push_into_position_map_or_insert(
                            1,
                            gate_number,
                            &mut position_map,
                            &right.value_string,
                        );
                        Self::push_into_position_map_or_insert(
                            2,
                            gate_number,
                            &mut position_map,
                            &result.value_string,
                        );
                        result
                    })
                    .unwrap()
            })
            .reduce(|pre, cur| {
                let mut gate_list = gate_list.borrow_mut();
                let mut gate_set = gate_set.borrow_mut();
                let mut position_map = position_map.borrow_mut();
                self.generate_additional_gate(
                    &mut gate_list,
                    &mut gate_set,
                    &mut position_map,
                    pre,
                    cur,
                )
            });

        (gate_list.take(), position_map.take())
    }

    /// Generate the circuit with a [gate_list] and [position_map] to coordinate pair accumulator for copy constraint
    fn gen_circuit(
        gate_list: Vec<ParserGate>,
        position_map: HashMap<String, Vec<(usize, usize)>>,
    ) -> Circuit {
        let mut result = Circuit::default();
        let mut position_map = position_map
            .into_iter()
            .map(|(key, mut vec)| {
                vec.reverse();
                vec.rotate_right(1);
                (key, vec)
            })
            .collect::<HashMap<String, Vec<(usize, usize)>>>();
        for gate in gate_list.iter() {
            #[cfg(test)]
            println!("{:?}", gate);
            let left = position_map
                .get_mut(&gate.left.value_string)
                .unwrap()
                .pop()
                .unwrap();
            let left = (left.0, left.1);
            let right = position_map
                .get_mut(&gate.right.value_string)
                .unwrap()
                .pop()
                .unwrap();
            let right = (right.0, right.1);
            let bottom = position_map
                .get_mut(&gate.bottom.value_string)
                .unwrap()
                .pop()
                .unwrap();
            let bottom = (bottom.0, bottom.1);
            match &gate.type_of_circuit {
                Addition => {
                    result = result.add_addition_gate(left, right, bottom, Fr::from(0));
                }
                Multiplication => {
                    result = result.add_multiplication_gate(left, right, bottom, Fr::from(0));
                }
                Constant => {
                    // result = result.add_constant_gate(left, right, bottom, Fr::from(0));
                }
            }
            #[cfg(test)]
            println!("{:?} {:?} {:?}", left, right, bottom);
        }
        result
    }

    /// Generate additional gate
    ///
    /// Take in [left] and [right] as corresponding wire and output result wire
    fn generate_additional_gate(
        &self,
        gate_list: &mut Vec<ParserGate>,
        gate_set: &mut HashSet<ParserGate>,
        position_map: &mut HashMap<String, Vec<(usize, usize)>>,
        left: ParserWire,
        right: ParserWire,
    ) -> ParserWire {
        let gate_number = gate_list.len();
        let result = ParserWire::new(
            format!("{}+{}", &left.value_string, &right.value_string),
        );
        let gate = ParserGate::new(left.clone(), right.clone(), result.clone(), Addition);
        //if this gate already exist, skip this move
        if gate_set.get(&gate).is_some() {
            return result;
        }
        gate_list.push(gate.clone());
        gate_set.insert(gate);

        Self::push_into_position_map_or_insert(0, gate_number, position_map, &left.value_string);
        Self::push_into_position_map_or_insert(1, gate_number, position_map, &right.value_string);
        Self::push_into_position_map_or_insert(2, gate_number, position_map, &result.value_string);
        result
    }

    /// Insert a pair of (x, y) corresponding to [wire_number] and [gate_number] into [position_map] by checking if it exists in the map or not
    //TODO: it could have been try_insert() or something but i think it should be in a wrapper instead
    fn push_into_position_map_or_insert(
        wire_number: usize,
        gate_number: usize,
        position_map: &mut HashMap<String, Vec<(usize, usize)>>,
        value: &str,
    ) {
        let var_exist = position_map.get(value).is_some();
        if var_exist {
            position_map
                .get_mut(value)
                .expect("var_exist guaranty its existence")
                .push((wire_number, gate_number))
        } else {
            position_map.insert(value.to_string(), vec![(wire_number, gate_number)]);
        }
    }

    /// Parse a polynomial string to be compatible with parser
    ///
    /// Feature:
    /// - Lower case
    /// - Expand simple ^ into *
    /// - Delete space character " "
    fn parse_string(string: &str) -> String {
        let string = string.to_lowercase();
        let mut result = String::new();
        let mut last_char = ' ';
        let mut flag = false;
        for char in string.chars() {
            if char == ' ' {
                continue;
            }
            if char == '^' {
                flag = true;
            } else if flag {
                if char.is_numeric() {
                    for _ in 0..char.to_string().parse::<i32>().unwrap() - 1 {
                        result.push('*');
                        result.push(last_char);
                    }
                    flag = false;
                } else {
                    panic!("can't parse polynomial")
                }
            } else {
                last_char = char;
                result.push(char);
            }
        }
        result
    }
}

//TODO: implement / operator

#[cfg(test)]
mod tests {
    use crate::common_preprocessed_input::parser::Parser;
    use crate::common_processed_input_const::COMMON_PROCESSED_INPUT;

    /// Test generated circuit with prover
    #[test]
    fn parser_prover_test() {
        let parser = Parser::default();
        let cpi = parser.parse("x*y+3*x^2+x*y*z=11");
        let a:[u8;1928] = [8, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 96, 255, 255, 255, 95, 127, 249, 254, 223, 129, 134, 86, 84, 3, 39, 5, 6, 5, 39, 4, 32, 77, 110, 2, 250, 147, 136, 116, 72, 23, 162, 228, 66, 41, 0, 194, 125, 26, 33, 115, 80, 208, 95, 188, 34, 237, 52, 199, 30, 68, 236, 29, 209, 207, 147, 235, 159, 16, 93, 210, 53, 0, 0, 0, 224, 255, 255, 255, 223, 127, 203, 255, 95, 128, 180, 119, 170, 0, 59, 52, 1, 1, 59, 103, 6, 169, 175, 51, 101, 234, 180, 125, 14, 234, 93, 27, 189, 213, 255, 61, 130, 228, 58, 139, 175, 50, 68, 1, 49, 24, 163, 218, 234, 195, 235, 27, 98, 120, 233, 177, 137, 66, 74, 27, 62, 0, 0, 0, 224, 255, 255, 255, 223, 127, 203, 255, 95, 128, 180, 119, 170, 0, 59, 52, 1, 1, 59, 103, 6, 169, 175, 51, 101, 234, 180, 125, 14, 234, 93, 27, 189, 213, 63, 62, 130, 228, 250, 11, 77, 51, 4, 2, 108, 76, 100, 91, 136, 247, 94, 112, 133, 120, 233, 177, 137, 66, 74, 27, 62, 0, 0, 0, 224, 255, 255, 255, 223, 127, 203, 255, 95, 128, 180, 119, 170, 0, 59, 52, 1, 1, 59, 103, 6, 169, 175, 51, 101, 234, 180, 125, 14, 23, 162, 228, 66, 41, 192, 193, 125, 26, 97, 242, 178, 207, 159, 187, 231, 184, 115, 70, 129, 16, 121, 201, 173, 207, 147, 235, 159, 16, 93, 210, 53, 8, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 96, 255, 255, 255, 95, 127, 249, 254, 223, 129, 134, 86, 84, 3, 39, 5, 6, 5, 39, 4, 32, 77, 110, 2, 250, 147, 136, 116, 72, 23, 162, 228, 66, 41, 0, 194, 125, 26, 33, 115, 80, 208, 95, 188, 34, 237, 52, 199, 30, 68, 236, 29, 209, 207, 147, 235, 159, 16, 93, 210, 53, 0, 0, 0, 224, 255, 255, 255, 223, 127, 203, 255, 95, 128, 180, 119, 170, 0, 59, 52, 1, 1, 59, 103, 6, 169, 175, 51, 101, 234, 180, 125, 14, 234, 93, 27, 189, 213, 255, 61, 130, 228, 58, 139, 175, 50, 68, 1, 49, 24, 163, 218, 234, 195, 235, 27, 98, 120, 233, 177, 137, 66, 74, 27, 62, 0, 0, 0, 224, 255, 255, 255, 223, 127, 203, 255, 95, 128, 180, 119, 170, 0, 59, 52, 1, 1, 59, 103, 6, 169, 175, 51, 101, 234, 180, 125, 14, 234, 93, 27, 189, 213, 63, 62, 130, 228, 250, 11, 77, 51, 4, 2, 108, 76, 100, 91, 136, 247, 94, 112, 133, 120, 233, 177, 137, 66, 74, 27, 62, 0, 0, 0, 224, 255, 255, 255, 223, 127, 203, 255, 95, 128, 180, 119, 170, 0, 59, 52, 1, 1, 59, 103, 6, 169, 175, 51, 101, 234, 180, 125, 14, 23, 162, 228, 66, 41, 192, 193, 125, 26, 97, 242, 178, 207, 159, 187, 231, 184, 115, 70, 129, 16, 121, 201, 173, 207, 147, 235, 159, 16, 93, 210, 53, 8, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 128, 255, 255, 255, 127, 255, 45, 255, 127, 1, 210, 222, 169, 2, 236, 208, 4, 4, 236, 156, 25, 164, 190, 206, 148, 169, 211, 246, 57, 251, 213, 44, 10, 47, 224, 248, 51, 4, 91, 168, 30, 251, 72, 127, 47, 125, 41, 40, 63, 12, 157, 105, 125, 237, 251, 153, 147, 56, 199, 139, 84, 1, 0, 0, 32, 255, 223, 255, 31, 127, 48, 62, 81, 130, 143, 197, 139, 106, 60, 173, 57, 109, 99, 40, 27, 159, 205, 105, 196, 104, 242, 111, 101, 17, 120, 17, 77, 89, 0, 187, 177, 31, 128, 221, 189, 200, 100, 254, 27, 255, 230, 13, 35, 226, 234, 247, 44, 117, 18, 232, 9, 246, 124, 112, 22, 1, 0, 0, 64, 255, 255, 255, 63, 255, 196, 254, 63, 2, 59, 206, 254, 3, 98, 57, 7, 6, 98, 107, 38, 246, 29, 54, 95, 126, 61, 242, 86, 6, 42, 211, 245, 207, 223, 6, 204, 250, 64, 213, 67, 7, 155, 61, 233, 83, 237, 248, 44, 200, 199, 123, 146, 90, 129, 3, 150, 26, 224, 97, 31, 1, 0, 0, 32, 255, 31, 0, 32, 127, 240, 190, 238, 130, 79, 198, 198, 158, 253, 45, 215, 160, 214, 124, 62, 159, 205, 105, 196, 104, 242, 111, 101, 240, 135, 238, 178, 165, 63, 69, 78, 223, 155, 161, 223, 58, 255, 191, 114, 58, 178, 20, 132, 89, 96, 150, 41, 211, 106, 181, 31, 93, 42, 125, 93, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 32, 255, 255, 255, 31, 127, 144, 254, 159, 130, 239, 69, 169, 4, 157, 109, 8, 7, 157, 210, 44, 159, 205, 105, 196, 104, 242, 111, 101, 240, 135, 238, 178, 165, 31, 69, 78, 223, 59, 225, 144, 58, 159, 63, 85, 160, 81, 84, 181, 191, 38, 236, 23, 211, 106, 181, 31, 93, 42, 125, 93, 0, 0, 0, 0, 0, 32, 0, 0, 0, 96, 192, 78, 0, 96, 128, 29, 154, 96, 192, 206, 153, 57, 170, 17, 0, 0, 0, 0, 0, 0, 0, 0, 6, 42, 211, 245, 207, 255, 6, 204, 250, 160, 149, 146, 7, 251, 189, 6, 238, 77, 185, 251, 97, 1, 38, 164, 90, 129, 3, 150, 26, 224, 97, 31, 0, 0, 0, 224, 255, 255, 255, 223, 127, 203, 255, 95, 128, 180, 119, 170, 0, 59, 52, 1, 1, 59, 103, 6, 169, 175, 51, 101, 234, 180, 125, 14, 17, 120, 17, 77, 89, 224, 186, 177, 31, 32, 29, 111, 200, 4, 126, 254, 100, 134, 77, 84, 72, 177, 77, 27, 117, 18, 232, 9, 246, 124, 112, 22, 1, 0, 0, 0, 255, 223, 255, 255, 254, 251, 61, 177, 2, 68, 61, 54, 107, 119, 225, 58, 110, 158, 143, 33, 72, 125, 157, 41, 83, 167, 237, 115, 251, 213, 44, 10, 47, 0, 249, 51, 4, 187, 104, 109, 251, 168, 255, 76, 23, 138, 232, 13, 166, 214, 19, 143, 237, 251, 153, 147, 56, 199, 139, 84, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 218, 235, 113, 68, 147, 94, 221, 194, 149, 166, 210, 23, 184, 34, 208, 16, 177, 106, 195, 1, 241, 80, 206, 28, 76, 233, 132, 109, 209, 134, 100, 82, 172, 155, 216, 53, 17, 0, 165, 2, 61, 153, 224, 182, 1, 144, 65, 41, 127, 58, 152, 206, 38, 250, 72, 64, 2, 232, 62, 64, 98, 243, 169, 106, 150, 5, 196, 155, 22, 158, 151, 105, 197, 147, 248, 99, 202, 178, 225, 140, 34, 42, 92, 75, 95, 156, 155, 249, 123, 251, 43, 82, 8, 98, 93, 22, 6, 54, 163, 94, 254, 61, 187, 112, 157, 51, 223, 99, 160, 198, 136, 137, 177, 198, 107, 45, 19, 89, 102, 6, 246, 135, 67, 18, 166, 21, 227, 29, 229, 51, 72, 71, 5, 32, 55, 54, 233, 39, 181, 235, 44, 123, 162, 241, 198, 161, 178, 194, 255, 15, 61, 233, 193, 38, 125, 136, 209, 61, 176, 88, 205, 139, 251, 79, 196, 128, 27, 230, 124, 235, 157, 237, 146, 7, 97, 45, 227, 158, 228, 74, 78, 202, 68, 199, 72, 78, 64, 191, 164, 25, 148, 93, 40, 26, 246, 239, 130, 224, 124, 143, 186, 222, 144, 159, 23, 71, 211, 33, 80, 10, 120, 111, 137, 236, 53, 217, 73, 151, 56, 122, 71, 187, 201, 32, 252, 225, 252, 50, 93, 62, 173, 152, 166, 228, 177, 254, 148, 221, 185, 36, 220, 120, 3, 66, 89, 164, 34, 254, 218, 97, 65, 218, 152, 102, 8, 112, 8, 0, 0, 0, 0, 0, 0, 0, 245, 171, 89, 212, 94, 96, 242, 39, 9, 209, 19, 135, 245, 54, 178, 243, 248, 211, 216, 128, 19, 248, 186, 44, 229, 217, 253, 199, 242, 80, 37, 82, 190, 19, 234, 162, 106, 32, 96, 212, 220, 173, 126, 99, 74, 160, 72, 184, 23, 71, 50, 191, 161, 227, 131, 120, 206, 74, 243, 228, 109, 187, 156, 114, 223, 15, 221, 37, 76, 223, 138, 92, 191, 146, 133, 107, 116, 227, 50, 63, 63, 36, 49, 109, 122, 11, 192, 97, 176, 183, 52, 224, 59, 23, 8, 100, 245, 171, 89, 84, 95, 96, 242, 167, 9, 163, 20, 7, 244, 100, 211, 73, 246, 231, 7, 124, 15, 12, 30, 19, 65, 27, 47, 51, 73, 125, 46, 24, 12, 84, 166, 107, 160, 31, 13, 24, 246, 115, 233, 125, 11, 132, 26, 149, 162, 11, 95, 75, 139, 131, 7, 179, 16, 68, 56, 151, 139, 236, 204, 4, 69, 236, 21, 221, 147, 159, 159, 171, 33, 28, 254, 126, 185, 21, 83, 10, 188, 187, 191, 177, 54, 109, 254, 176, 29, 241, 120, 217, 142, 191, 71, 59, 34, 240, 34, 154, 178, 160, 117, 99, 63, 224, 121, 143, 144, 169, 123, 223, 47, 172, 218, 217, 246, 40, 241, 36, 234, 36, 208, 19, 236, 249, 224, 44, 12, 84, 166, 43, 160, 223, 13, 216, 245, 74, 107, 22, 14, 45, 12, 155, 64, 197, 73, 38, 40, 83, 211, 41, 99, 163, 159, 97, 96, 86, 200, 33, 8, 0, 0, 0, 0, 0, 0, 0, 51, 104, 52, 231, 11, 65, 48, 21, 95, 64, 22, 97, 88, 238, 248, 162, 96, 113, 167, 144, 11, 103, 234, 28, 95, 55, 184, 29, 226, 118, 81, 67, 162, 89, 106, 231, 182, 221, 165, 97, 59, 51, 165, 6, 89, 75, 247, 213, 48, 214, 178, 220, 83, 97, 6, 82, 54, 239, 151, 78, 126, 3, 168, 41, 228, 51, 72, 199, 5, 128, 55, 182, 233, 25, 247, 87, 44, 201, 68, 160, 146, 215, 34, 42, 201, 208, 158, 4, 30, 104, 174, 243, 39, 106, 185, 30, 250, 213, 44, 138, 47, 96, 249, 179, 4, 173, 170, 217, 250, 246, 161, 251, 226, 191, 88, 117, 111, 151, 117, 170, 73, 61, 203, 254, 142, 243, 148, 26, 52, 116, 4, 16, 58, 127, 228, 121, 1, 106, 95, 242, 241, 34, 179, 122, 37, 96, 194, 196, 190, 52, 249, 139, 76, 157, 95, 100, 66, 22, 206, 94, 229, 57, 176, 91, 28, 31, 145, 104, 186, 92, 153, 101, 121, 53, 255, 63, 143, 184, 255, 141, 63, 61, 26, 143, 184, 217, 208, 171, 129, 141, 110, 102, 85, 88, 87, 129, 190, 193, 166, 120, 158, 192, 210, 23, 235, 55, 247, 80, 199, 193, 196, 17, 55, 35, 131, 189, 73, 92, 136, 49, 206, 112, 50, 112, 17, 126, 121, 161, 112, 159, 20, 36, 241, 253, 128, 139, 19, 150, 234, 188, 247, 101, 177, 127, 82, 245, 7, 145, 25, 102, 212, 98, 209, 98, 51, 7];
        assert_eq!(Vec::<u8>::from(COMMON_PROCESSED_INPUT), a);
    }

}
