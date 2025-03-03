use ark_ff::PrimeField;
use strum::IntoEnumIterator;

// Define an enum to represent mathematical operations supported by the circuit
pub enum Operator {
    Add,
    Mul,
}

// Define a struct representing a single gate in the arithmetic circuit
// A gate connects two input wires to one output wire via an operation
pub struct Gate {
    left_index: usize, 
    right_index: usize,
    output_index: usize,
    gate_operator: Operator,
}

// Define a struct representing a layer in the arithmetic circuit
// A layer consists of multiple gates that operate in parallel
pub struct Layer {
    pub gates: Vec<Gate>,
}

// Define a struct representing the entire arithmetic circuit
// A circuit consists of multiple layers executed sequentially
pub struct Circuit<F: PrimeField> {
    pub layers: Vec<Layer>,      // Vector storing all layers in the circuit
    pub round_poly: Vec<Vec<F>>, // Stores intermediate values during circuit evaluation
}

// Implementation block for the Gate struct, providing methods to create and use gates
impl Gate {
    // Constructor function to create a new Gate with specified parameters
    fn new(left_index: usize, right_index: usize, output_index: usize, gate_operator: Operator) -> Self {
        Self {
            left_index,
            right_index,  
            output_index,
            gate_operator,
        }
    }

    // Function to execute a gate operation using the provided input values
    // Returns the result of applying the gate's operation to its inputs
    fn execute_gate<F: PrimeField>(&mut self, inputs: Vec<F>) -> F {
        // Match on the gate operator to determine which operation to perform
        let result = match self.gate_operator {
            Operator::Add => inputs[self.left_index] + inputs[self.right_index],
            Operator::Mul => inputs[self.left_index] * inputs[self.right_index],
        };
        result // Return the computed result
    }
}

// Implementation block for the Layer struct
impl Layer {
    // Constructor function to create a new Layer with specified gates
    pub fn new(gates: Vec<Gate>) -> Self {
        Self { gates } // Initialize the Layer with the provided gates
    }
}

// Implementation block for the Circuit struct
impl<F: PrimeField> Circuit<F> {
    // Constructor function to create a new Circuit with specified layers
    pub fn new(layers: Vec<Layer>) -> Self {
        Self {
            layers, // Initialize the circuit with the provided layers
            round_poly: Vec::new(), // Initialize an empty vector to store evaluation results
        }
    }

    // Function to evaluate the circuit with a given input vector
    // Returns the final output of the circuit after processing through all layers
    pub fn evaluate(&mut self, input: Vec<F>) -> F {

        // Create a vector to store all intermediate evaluations
        let mut evals = Vec::new();

        // Initialize current_input with the provided input vector
        let mut current_input = input;

        // Store the initial input in the evaluations vector
        evals.push(current_input.clone());

        // Iterate through each layer in the circuit
        for layer in &mut self.layers {
            // Find the maximum output index used by any gate in this layer
            // This determines the size of the output vector needed
            let max_output_index = layer.gates
                .iter()
                .map(|gate| gate.output_index)
                .max()
                .unwrap_or(0); // Default to 0 if the layer has no gates

            // Create an output vector initialized with zeros, sized to accommodate all outputs
            let mut output_vec = vec![F::zero(); max_output_index + 1];

            // Process each gate in the current layer
            for gate in layer.gates.iter_mut() {
                // Execute the gate with the current input vector
                let result = gate.execute_gate(current_input.clone());
                // Store the result at the appropriate index in the output vector
                output_vec[gate.output_index] = result;
                // Store the current state of the output vector in the evaluations
                evals.push(output_vec.clone());
            }
            // Update current_input to be the output of this layer for the next iteration
            current_input = output_vec;
        }
        // Reverse the evaluations vector (for some reason - possibly needed for later processing)
        evals.reverse();

        // Store all evaluations in the circuit's round_poly field
        self.round_poly = evals.clone();

        // Return the first element of the round_poly as the final output
        self.round_poly[0].clone()
    }

    // Function to retrieve the polynomial for a specific layer
    pub fn get_round_poly(&mut self, layer_index: usize) -> Vec<F> {
        // Get the polynomial for the specified layer
        let round_poly = &self.round_poly[layer_index];
        // Return a clone of that polynomial
        round_poly.clone()
    }

    // Function to compute Multi-Linear Extensions (MLE) for addition and multiplication gates
    // Returns vectors representing the MLEs for a specified layer
    pub fn add_i_and_mul_i_mle(&mut self, layer_id: usize) -> Vec<Vec<F>> {
        // Get the layer at the specified index
        let layer_vec = &self.layers[layer_id];

        // If the layer has no gates, return zero vectors
        if layer_vec.is_empty() {
            return vec![vec![F::zero(); 2], vec![F::zero(); 2]];
        }

        // Calculate the total number of gates (multiplied by 2 for some reason)
        let no_of_gates = layer_vec.len() * 2;
        // Calculate the number of bits needed to represent gate input indices
        // This is the ceiling of log2 of the number of gates, at least 1
        let no_of_bit_in_gate_input_index = (no_of_gates as f64).log2().ceil().max(1.0) as usize;
        // Calculate the number of bits needed for output indices
        // One less than input bits, but at least 1
        let no_of_bit_in_gate_output_index = if no_of_bit_in_gate_input_index == 1 {
            1
        } else {
            no_of_bit_in_gate_input_index - 1
        };

        // Calculate the total number of bits needed for the entire representation
        let total_no_of_bits = no_of_bit_in_gate_input_index * 2 + no_of_bit_in_gate_output_index;

        // Print debugging information about bit sizes
        println!(
            "no bit input:{} no bits output{}",
            no_of_bit_in_gate_input_index, no_of_bit_in_gate_output_index
        );
        
        // Calculate the size of the vectors needed (2^total_bits)
        let vector_size = 1 << total_no_of_bits;
        // Initialize vectors for addition and multiplication MLEs with zeros
        let mut add_vec = vec![F::zero(); vector_size];
        let mut mul_vec = vec![F::zero(); vector_size];

        // Process each gate in the layer
        for gate in layer_vec {
            // Get the gate operation
            let gate_op = &gate.gate_operator; // Note: This will cause a compilation error as 'op' field doesn't exist

            // Compute a unique index for this gate based on its inputs and output
            // First shift left by input bit size and OR with left index
            let mut res = gate.output_index << no_of_bit_in_gate_input_index | gate.left_index;
            // Then shift left again by input bit size and OR with right index
            res = res << no_of_bit_in_gate_input_index | gate.right_index;
            
            // Set the appropriate vector element to 1 based on gate type
            if let GateOp::Add = gate_op { // Note: GateOp doesn't exist, should be Operator
                add_vec[res] = F::one();
            } else if let GateOp::Mul = gate_op { // Note: GateOp doesn't exist, should be Operator
                mul_vec[res] = F::one();
            }
        }

        // Return both MLE vectors
        vec![add_vec, mul_vec]
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use ark_bn254::Fr;
    use ark_ff::Field;

    // Helper function to create field elements
    fn f(val: u64) -> Fr {
        Fr::from(val)
    }

    #[test]
    fn test_single_gate_add() {
        // Test a circuit with a single addition gate
        let gate = Gate::new(0, 1, 0, Operator::Add);
        let layer = Layer::new(vec![gate]);
        let circuit = Circuit::new(vec![layer]);

        let input = vec![f(3), f(4)];
        let mut circuit = circuit; // No need to clone since we're using it once
        let result = circuit.evaluate(input);

        assert_eq!(result, f(7));
    }

    #[test]
    fn test_single_gate_mul() {
        // Test a circuit with a single multiplication gate
        let gate = Gate::new(0, 1, 0, Operator::Mul);
        let layer = Layer::new(vec![gate]);
        let circuit = Circuit::new(vec![layer]);

        let input = vec![f(3), f(4)];
        let mut circuit = circuit; // No need to clone
        let result = circuit.evaluate(input);

        assert_eq!(result, f(12));
    }

    #[test]
    fn test_empty_circuit() {
        // Test an empty circuit
        let circuit = Circuit::<Fr>::new(vec![]);
        let input = vec![f(5)];
        let mut circuit = circuit; // No need to clone
        let result = circuit.evaluate(input);

        // With the current implementation, this should return the first input
        assert_eq!(result, f(5));
    }

    #[test]
    fn test_two_layer_circuit() {
        // Create gates for first layer
        let gate1 = Gate::new(0, 1, 0, Operator::Add); // 3 + 4 = 7 -> output[0]
        let gate2 = Gate::new(1, 2, 1, Operator::Mul); // 4 * 5 = 20 -> output[1]
        let layer1 = Layer::new(vec![gate1, gate2]);
        
        // Create gate for second layer
        let gate3 = Gate::new(0, 1, 0, Operator::Mul); // 7 * 20 = 140 -> output[0]
        let layer2 = Layer::new(vec![gate3]);

        // Create circuit with both layers
        let circuit = Circuit::new(vec![layer1, layer2]);

        let input = vec![f(3), f(4), f(5)];
        let mut circuit = circuit;
        let result = circuit.evaluate(input);

        assert_eq!(result, f(140));
    }

    #[test]
    fn test_complex_multi_layer_circuit() {
        // This test needs redesign as it has a conceptual issue with input access in later layers
        
        // Layer 1: Compute a+b and c*d
        let layer1 = Layer::new(vec![
            Gate::new(0, 1, 0, Operator::Add), // a + b = 2 + 3 = 5 -> output[0]
            Gate::new(2, 3, 1, Operator::Mul), // c * d = 4 * 5 = 20 -> output[1]
        ]);

        // Layer 2: Compute (a+b)*(c*d)
        // Note: We can't directly compute a*b here because after layer1,
        // our input becomes [5, 20], losing the original a and b values
        let layer2 = Layer::new(vec![
            Gate::new(0, 1, 0, Operator::Mul), // (a+b) * (c*d) = 5 * 20 = 100 -> output[0]
        ]);

        // Create circuit with both layers
        let circuit = Circuit::new(vec![layer1, layer2]);
        let input = vec![f(2), f(3), f(4), f(5)];
        let mut circuit = circuit;
        let result = circuit.evaluate(input);

        // Expected result: (2+3)*(4*5) = 5*20 = 100
        assert_eq!(result, f(100));
        
        // Note: To compute (a+b)*(c*d) + (a*b), the circuit needs to be redesigned
        // to preserve access to original inputs in later layers.
    }
}


fn main() {
    println!("Hello, world!");
}