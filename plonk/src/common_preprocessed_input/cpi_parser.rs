use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fs;

use ark_bls12_381::Fr;
use ark_poly::univariate::DensePolynomial;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};

use crate::common_preprocessed_input::cpi_circuit::CPICircuit;
use crate::common_preprocessed_input::cpi_parser::TypeOfCircuit::{
    Addition, Constant, Multiplication,
};
use crate::compiled_circuit::CompiledCircuit;
use crate::constraint::{CopyConstraints, GateConstraints};

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
/// Enum defining circuit type
enum TypeOfCircuit {
    Addition,
    Multiplication,
    Constant,
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
    fn new(
        left: ParserWire,
        right: ParserWire,
        bottom: ParserWire,
        type_of_circuit: TypeOfCircuit,
    ) -> Self {
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
pub struct CommonPreprocessedInput {
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

/// String to common preprocessed input parser
///
/// See parse function for usage
#[derive(Default)]
pub struct CPIParser {}

impl CPIParser {
    /// Compute common_preprocessed_input and
    /// deserialize it into vec<u8>
    pub fn compute_common_preprocessed_input(self, input: &str) -> Result<CommonPreprocessedInput, String> {
        let input = Self::normalize(input);

        //Step 1: prepare gate_list and position_map prior to coordinate pair accumulator
        let (gate_list, position_map) = self.prepare_generation(&input);

        //Step 2: generate the actual circuit with coordinate pair accumulator for copy constraint
        let circuit = Self::gen_circuit(gate_list, position_map);

        Ok(CommonPreprocessedInput::new(circuit.compile()?))
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
                    .map(|s| {
                        self.check_constant(
                            &mut gate_list.borrow_mut(),
                            &mut gate_set.borrow_mut(),
                            &mut position_map.borrow_mut(),
                            s.clone(),
                        );
                        ParserWire::new(s.clone())
                    })
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
                        let gate = ParserGate::new(
                            left.clone(),
                            right.clone(),
                            result.clone(),
                            Multiplication,
                        );
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
    ) -> CPICircuit {
        let mut result = CPICircuit::default();
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
                    result = result.add_constant_gate(
                        left,
                        right,
                        bottom,
                        Fr::from(gate.clone().left.value_string.parse::<i32>().unwrap()),
                        Fr::from(0),
                    );
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
        let result = ParserWire::new(format!("{}+{}", &left.value_string, &right.value_string));
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

    /// Generate constant gate
    ///
    /// Take in `value` to make constant gate.
    /// Constant gate ensure the prover send the correct polynomial
    fn generate_constant_gate(
        &self,
        gate_list: &mut Vec<ParserGate>,
        gate_set: &mut HashSet<ParserGate>,
        position_map: &mut HashMap<String, Vec<(usize, usize)>>,
        value: ParserWire,
    ) -> ParserWire {
        let gate_number = gate_list.len();
        let right = ParserWire::new("0".to_string());
        let result = ParserWire::new(format!("{}+{}", &value.value_string, "0"));
        let gate = ParserGate::new(value.clone(), right.clone(), result.clone(), Constant);
        //if this gate already exist, skip this move
        if gate_set.get(&gate).is_some() {
            return result;
        }
        gate_list.push(gate.clone());
        gate_set.insert(gate);

        Self::push_into_position_map_or_insert(0, gate_number, position_map, &value.value_string);
        Self::push_into_position_map_or_insert(1, gate_number, position_map, "0");
        Self::push_into_position_map_or_insert(2, gate_number, position_map, &result.value_string);
        result
    }

    fn check_constant(
        &self,
        gate_list: &mut Vec<ParserGate>,
        gate_set: &mut HashSet<ParserGate>,
        position_map: &mut HashMap<String, Vec<(usize, usize)>>,
        value: String,
    ) {
        match value.parse::<i32>() {
            Ok(number) => {
                self.generate_constant_gate(
                    gate_list,
                    gate_set,
                    position_map,
                    ParserWire::new(value),
                );
            }
            Err(err) => {}
        };
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
    fn normalize(string: &str) -> String {
        let string = string.to_lowercase();
        let mut result = String::new();
        let mut last_char = ' ';
        let mut number_buffer = String::new();
        let mut flag = false;
        for char in string.chars() {
            if char == ' ' {
                continue;
            }
            if char == '^' {
                flag = true;
            } else if !char.is_numeric() {
                if flag {
                    if !number_buffer.is_empty() {
                        for _ in 0..number_buffer.parse::<i32>().unwrap() - 1 {
                            result.push('*');
                            result.push(last_char);
                        }
                        flag = false;
                    } else {
                        panic!("can't parse polynomial")
                    }
                }
                last_char = char;
                result.push(char);
                number_buffer = String::new();
            } else {
                number_buffer.push(char);
                if !flag {
                    last_char = char;
                    result.push(char);
                }
            }
        }
        if flag && !number_buffer.is_empty() {
            for _ in 0..number_buffer.parse::<i32>().unwrap() - 1 {
                result.push('*');
                result.push(last_char);
            }
        }
        result
    }
}

//TODO: implement / operator

#[cfg(test)]
mod tests {
    use ark_bls12_381::Fr;
    use ark_serialize::CanonicalDeserialize;

    use crate::common_preprocessed_input::cpi_parser::{CPIParser, CommonPreprocessedInput};
    use crate::parser::Parser;

    // /// Test generated circuit with prover circuit
    // #[test]
    // fn parser_prover_test() {
    //     let str = "x*y+3*x^2+x*y*z=11";
    //
    //     // Common preprocessed input parser
    //     CPIParser::default().parse_cpi_into_file(str);
    //     let vec = Vec::<u8>::from(COMMON_PREPROCESSED_INPUT);
    //     let cpi = CommonPreprocessedInput::deserialize_compressed(&vec[..]).unwrap();
    //
    //     // Prover parser
    //     let mut parser = Parser::default();
    //     parser.add_witness("x", Fr::from(1));
    //     parser.add_witness("y", Fr::from(2));
    //     parser.add_witness("z", Fr::from(3));
    //     let compiled_circuit = parser.parse(str).compile().unwrap();
    //     let copy_constraint = compiled_circuit.copy_constraints();
    //     let gate_constraint = compiled_circuit.gate_constraints();
    //
    //     assert_eq!(cpi.n, compiled_circuit.size);
    //     assert_eq!(cpi.k1, copy_constraint.k1().clone());
    //     assert_eq!(cpi.k2, copy_constraint.k2().clone());
    //     assert_eq!(cpi.q_lx, gate_constraint.q_lx().clone());
    //     assert_eq!(cpi.q_rx, gate_constraint.q_rx().clone());
    //     assert_eq!(cpi.q_mx, gate_constraint.q_mx().clone());
    //     assert_eq!(cpi.q_ox, gate_constraint.q_ox().clone());
    //     assert_eq!(cpi.q_cx, gate_constraint.q_cx().clone());
    //     assert_eq!(cpi.s_sigma_1, copy_constraint.get_s_sigma_1().clone());
    //     assert_eq!(cpi.s_sigma_2, copy_constraint.get_s_sigma_2().clone());
    //     assert_eq!(cpi.s_sigma_3, copy_constraint.get_s_sigma_3().clone());
    //     assert_eq!(cpi.pi_x, gate_constraint.pi_x().clone());
    // }
}
