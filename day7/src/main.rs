use std::fs::File;
use std::io::prelude::*;
use std::io::{self};

use permute::permutations_of;

enum ParameterMode {
    Position,
    Immediate,
}

enum Operation {
    Addition(ParameterMode, ParameterMode, ParameterMode),
    Multiplication(ParameterMode, ParameterMode, ParameterMode),
    Input(ParameterMode),
    Output(ParameterMode),
    JumpTrue(ParameterMode, ParameterMode),
    JumpFalse(ParameterMode, ParameterMode),
    Less(ParameterMode, ParameterMode, ParameterMode),
    Equal(ParameterMode, ParameterMode, ParameterMode),
    Terminate,
}

#[derive(PartialEq)]
enum ExitStatus {
    AwaitingInput(usize),
    Terminated,
}

fn main() -> Result<(), io::Error> {
    let mut file = File::open("input.txt")?;
    let mut program = String::new();
    file.read_to_string(&mut program)?;

    let program = program
        .trim()
        .split(',')
        .map(|i| i.parse::<i64>().unwrap())
        .collect::<Vec<_>>();

    let max = permutations_of(&[0, 1, 2, 3, 4])
        .map(permutation_to_array)
        .map(|permutation| amplifier_sequence(&program, &permutation))
        .max()
        .unwrap();

    println!("{}", max);

    let max = permutations_of(&[5, 6, 7, 8, 9])
        .map(permutation_to_array)
        .map(|permutation| amplifier_sequence_loop(&program, &permutation))
        .max()
        .unwrap();

    println!("{}", max);

    Ok(())
}

fn permutation_to_array<'a, I>(mut permutation: I) -> [u32; 5]
where
    I: Iterator<Item = &'a u32>,
{
    let mut permutation_array: [u32; 5] = Default::default();
    for i in 0..5 {
        permutation_array[i] = *permutation.next().unwrap();
    }
    permutation_array
}
fn amplifier_sequence(input: &[i64], phase_settings: &[u32; 5]) -> i64 {
    let mut input_signal = String::from("0");
    for phase in phase_settings.iter() {
        let mut program = vec![0; input.len()];
        program.copy_from_slice(&input);

        let input = format!("{}\n{}", phase.to_string(), input_signal);
        let mut output = Vec::new();

        compute(&mut program, input.as_bytes(), &mut output, 0);

        input_signal = String::from_utf8(output).unwrap();
    }

    input_signal
        .trim()
        .parse()
        .expect("Unexpected final amplifier value")
}

fn amplifier_sequence_loop(input: &[i64], phase_settings: &[u32; 5]) -> i64 {
    let mut input_signal = String::from("0");
    let mut exit_status: ExitStatus = ExitStatus::AwaitingInput(0);

    let mut programs = (0..5)
        .map(|_| {
            let mut program = vec![0; input.len()];
            program.copy_from_slice(&input);
            program
        })
        .collect::<Vec<_>>();
    let mut instruction_ptrs = vec![0; 5];

    let mut first = true;
    while exit_status != ExitStatus::Terminated {
        for (i, phase) in phase_settings.iter().enumerate() {
            let input = if first {
                format!("{}\n{}", phase.to_string(), input_signal)
            } else {
                input_signal
            };
            let mut output = Vec::new();

            exit_status = compute(
                &mut programs[i],
                input.as_bytes(),
                &mut output,
                instruction_ptrs[i],
            );

            input_signal = String::from_utf8(output).unwrap();

            if let ExitStatus::AwaitingInput(ptr) = exit_status {
                instruction_ptrs[i] = ptr;
            }
        }
        first = false;
    }

    input_signal
        .trim()
        .parse()
        .expect("Unexpected final amplifier value")
}

fn compute<R, W>(
    input: &mut [i64],
    mut reader: R,
    mut writer: W,
    mut instruction_ptr: usize,
) -> ExitStatus
where
    R: BufRead,
    W: Write,
{
    let mut advance;
    loop {
        match parse_operation(input[instruction_ptr]) {
            Operation::Addition(p1, p2, _) => {
                advance = 4;
                input[input[instruction_ptr + 3] as usize] =
                    value_from_parameter(input, instruction_ptr + 1, p1)
                        + value_from_parameter(input, instruction_ptr + 2, p2);
            }
            Operation::Multiplication(p1, p2, _) => {
                advance = 4;
                input[input[instruction_ptr + 3] as usize] =
                    value_from_parameter(input, instruction_ptr + 1, p1)
                        * value_from_parameter(input, instruction_ptr + 2, p2);
            }
            Operation::Input(_) => {
                advance = 2;
                let mut op_input = String::new();
                reader.read_line(&mut op_input).expect("Unable to read");
                let op_input = op_input.trim().parse::<i64>();
                match op_input {
                    Ok(i) => input[input[instruction_ptr + 1] as usize] = i,
                    Err(_) => return ExitStatus::AwaitingInput(instruction_ptr),
                }
            }
            Operation::Output(p1) => {
                advance = 2;
                writeln!(
                    &mut writer,
                    "{}",
                    value_from_parameter(input, instruction_ptr + 1, p1)
                )
                .expect("Unable to write");
            }
            Operation::JumpTrue(p1, p2) => {
                let compare = value_from_parameter(input, instruction_ptr + 1, p1);
                match compare != 0 {
                    true => {
                        advance = 0;
                        instruction_ptr =
                            value_from_parameter(input, instruction_ptr + 2, p2) as usize;
                    }
                    false => advance = 3,
                }
            }
            Operation::JumpFalse(p1, p2) => {
                let compare = value_from_parameter(input, instruction_ptr + 1, p1);
                match compare == 0 {
                    true => {
                        advance = 0;
                        instruction_ptr =
                            value_from_parameter(input, instruction_ptr + 2, p2) as usize;
                    }
                    false => advance = 3,
                }
            }
            Operation::Less(p1, p2, _) => {
                advance = 4;
                input[input[instruction_ptr + 3] as usize] =
                    match value_from_parameter(input, instruction_ptr + 1, p1)
                        < value_from_parameter(input, instruction_ptr + 2, p2)
                    {
                        true => 1,
                        false => 0,
                    };
            }
            Operation::Equal(p1, p2, _) => {
                advance = 4;
                input[input[instruction_ptr + 3] as usize] =
                    match value_from_parameter(input, instruction_ptr + 1, p1)
                        == value_from_parameter(input, instruction_ptr + 2, p2)
                    {
                        true => 1,
                        false => 0,
                    };
            }
            Operation::Terminate => return ExitStatus::Terminated,
        }
        instruction_ptr += advance;
    }
}

fn parse_operation(instruction: i64) -> Operation {
    let op_code = instruction % 100;
    match op_code {
        1 => Operation::Addition(
            parse_parameter_mode(instruction, 0),
            parse_parameter_mode(instruction, 1),
            parse_parameter_mode(instruction, 2),
        ),
        2 => Operation::Multiplication(
            parse_parameter_mode(instruction, 0),
            parse_parameter_mode(instruction, 1),
            parse_parameter_mode(instruction, 2),
        ),
        3 => Operation::Input(parse_parameter_mode(instruction, 0)),
        4 => Operation::Output(parse_parameter_mode(instruction, 0)),
        5 => Operation::JumpTrue(
            parse_parameter_mode(instruction, 0),
            parse_parameter_mode(instruction, 1),
        ),
        6 => Operation::JumpFalse(
            parse_parameter_mode(instruction, 0),
            parse_parameter_mode(instruction, 1),
        ),
        7 => Operation::Less(
            parse_parameter_mode(instruction, 0),
            parse_parameter_mode(instruction, 1),
            parse_parameter_mode(instruction, 2),
        ),
        8 => Operation::Equal(
            parse_parameter_mode(instruction, 0),
            parse_parameter_mode(instruction, 1),
            parse_parameter_mode(instruction, 2),
        ),
        99 => Operation::Terminate,
        _ => panic!("Unrecognized instruction"),
    }
}

fn parse_parameter_mode(instruction: i64, position: u32) -> ParameterMode {
    match instruction / (10i64.pow(position + 2)) % 10 {
        0 => ParameterMode::Position,
        1 => ParameterMode::Immediate,
        _ => panic!("Unrecognized parameter mode"),
    }
}

fn value_from_parameter(input: &[i64], ptr: usize, mode: ParameterMode) -> i64 {
    match mode {
        ParameterMode::Position => input[input[ptr] as usize],
        ParameterMode::Immediate => input[ptr],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_computer() {
        let input = b"";
        let mut output = Vec::new();
        let mut program = [1, 9, 10, 3, 2, 3, 11, 0, 99, 30, 40, 50];
        compute(&mut program, &input[..], &mut output, 0);
        assert_eq!(program, [3500, 9, 10, 70, 2, 3, 11, 0, 99, 30, 40, 50]);

        let mut program = [1, 0, 0, 0, 99];
        compute(&mut program, &input[..], &mut output, 0);
        assert_eq!(program, [2, 0, 0, 0, 99]);

        let mut program = [2, 3, 0, 3, 99];
        compute(&mut program, &input[..], &mut output, 0);
        assert_eq!(program, [2, 3, 0, 6, 99]);

        let mut program = [2, 4, 4, 5, 99, 0];
        compute(&mut program, &input[..], &mut output, 0);
        assert_eq!(program, [2, 4, 4, 5, 99, 9801]);

        let mut program = [1, 1, 1, 4, 99, 5, 6, 0, 99];
        compute(&mut program, &input[..], &mut output, 0);
        assert_eq!(program, [30, 1, 1, 4, 2, 5, 6, 0, 99]);

        let mut program = [1002, 4, 3, 4, 33];
        compute(&mut program, &input[..], &mut output, 0);
        assert_eq!(program, [1002, 4, 3, 4, 99]);

        let input = b"8";
        let mut output = Vec::new();
        let mut program = [3, 9, 8, 9, 10, 9, 4, 9, 99, -1, 8];
        compute(&mut program, &input[..], &mut output, 0);
        assert_eq!(output, b"1\n");

        let input = b"9";
        let mut output = Vec::new();
        let mut program = [3, 9, 8, 9, 10, 9, 4, 9, 99, -1, 8];
        compute(&mut program, &input[..], &mut output, 0);
        assert_eq!(output, b"0\n");

        let input = b"8";
        let mut output = Vec::new();
        let mut program = [3, 9, 7, 9, 10, 9, 4, 9, 99, -1, 8];
        compute(&mut program, &input[..], &mut output, 0);
        assert_eq!(output, b"0\n");

        let input = b"7";
        let mut output = Vec::new();
        let mut program = [3, 9, 7, 9, 10, 9, 4, 9, 99, -1, 8];
        compute(&mut program, &input[..], &mut output, 0);
        assert_eq!(output, b"1\n");

        let input = b"8";
        let mut output = Vec::new();
        let mut program = [3, 3, 1108, -1, 8, 3, 4, 3, 99];
        compute(&mut program, &input[..], &mut output, 0);
        assert_eq!(output, b"1\n");

        let input = b"9";
        let mut output = Vec::new();
        let mut program = [3, 3, 1108, -1, 8, 3, 4, 3, 99];
        compute(&mut program, &input[..], &mut output, 0);
        assert_eq!(output, b"0\n");

        let input = b"8";
        let mut output = Vec::new();
        let mut program = [3, 3, 1107, -1, 8, 3, 4, 3, 99];
        compute(&mut program, &input[..], &mut output, 0);
        assert_eq!(output, b"0\n");

        let input = b"7";
        let mut output = Vec::new();
        let mut program = [3, 3, 1107, -1, 8, 3, 4, 3, 99];
        compute(&mut program, &input[..], &mut output, 0);
        assert_eq!(output, b"1\n");

        let input = b"2";
        let mut output = Vec::new();
        let mut program = [3, 12, 6, 12, 15, 1, 13, 14, 13, 4, 13, 99, -1, 0, 1, 9];
        compute(&mut program, &input[..], &mut output, 0);
        assert_eq!(output, b"1\n");

        let input = b"0";
        let mut output = Vec::new();
        let mut program = [3, 12, 6, 12, 15, 1, 13, 14, 13, 4, 13, 99, -1, 0, 1, 9];
        compute(&mut program, &input[..], &mut output, 0);
        assert_eq!(output, b"0\n");

        let input = b"2";
        let mut output = Vec::new();
        let mut program = [3, 3, 1105, -1, 9, 1101, 0, 0, 12, 4, 12, 99, 1];
        compute(&mut program, &input[..], &mut output, 0);
        assert_eq!(output, b"1\n");

        let input = b"0";
        let mut output = Vec::new();
        let mut program = [3, 3, 1105, -1, 9, 1101, 0, 0, 12, 4, 12, 99, 1];
        compute(&mut program, &input[..], &mut output, 0);
        assert_eq!(output, b"0\n");

        let program = [
            3, 21, 1008, 21, 8, 20, 1005, 20, 22, 107, 8, 21, 20, 1006, 20, 31, 1106, 0, 36, 98, 0,
            0, 1002, 21, 125, 20, 4, 20, 1105, 1, 46, 104, 999, 1105, 1, 46, 1101, 1000, 1, 20, 4,
            20, 1105, 1, 46, 98, 99,
        ];

        let input = b"7";
        let mut output = Vec::new();
        compute(&mut program.clone(), &input[..], &mut output, 0);
        assert_eq!(output, b"999\n");

        let input = b"8";
        let mut output = Vec::new();
        compute(&mut program.clone(), &input[..], &mut output, 0);
        assert_eq!(output, b"1000\n");

        let input = b"9";
        let mut output = Vec::new();
        compute(&mut program.clone(), &input[..], &mut output, 0);
        assert_eq!(output, b"1001\n");
    }

    #[test]
    fn test_amplifier_sequence() {
        let program = [
            3, 15, 3, 16, 1002, 16, 10, 16, 1, 16, 15, 15, 4, 15, 99, 0, 0,
        ];
        assert_eq!(amplifier_sequence(&program, &[4, 3, 2, 1, 0]), 43210);

        let program = [
            3, 23, 3, 24, 1002, 24, 10, 24, 1002, 23, -1, 23, 101, 5, 23, 23, 1, 24, 23, 23, 4, 23,
            99, 0, 0,
        ];
        assert_eq!(amplifier_sequence(&program, &[0, 1, 2, 3, 4]), 54321);

        let program = [
            3, 31, 3, 32, 1002, 32, 10, 32, 1001, 31, -2, 31, 1007, 31, 0, 33, 1002, 33, 7, 33, 1,
            33, 31, 31, 1, 32, 31, 31, 4, 31, 99, 0, 0, 0,
        ];
        assert_eq!(amplifier_sequence(&program, &[1, 0, 4, 3, 2]), 65210);
    }

    #[test]
    fn test_amplifier_sequence_loop() {
        let program = [
            3, 26, 1001, 26, -4, 26, 3, 27, 1002, 27, 2, 27, 1, 27, 26, 27, 4, 27, 1001, 28, -1,
            28, 1005, 28, 6, 99, 0, 0, 5,
        ];
        assert_eq!(
            amplifier_sequence_loop(&program, &[9, 8, 7, 6, 5]),
            139629729
        );

        let program = [
            3, 52, 1001, 52, -5, 52, 3, 53, 1, 52, 56, 54, 1007, 54, 5, 55, 1005, 55, 26, 1001, 54,
            -5, 54, 1105, 1, 12, 1, 53, 54, 53, 1008, 54, 0, 55, 1001, 55, 1, 55, 2, 53, 55, 53, 4,
            53, 1001, 56, -1, 56, 1005, 56, 6, 99, 0, 0, 0, 0, 10,
        ];
        assert_eq!(amplifier_sequence_loop(&program, &[9, 7, 8, 5, 6]), 18216);
    }
}
