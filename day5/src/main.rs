use std::fs::File;
use std::io::prelude::*;
use std::io::{self};

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

fn main() -> Result<(), io::Error> {
    let mut file = File::open("input.txt")?;
    let mut program = String::new();
    file.read_to_string(&mut program)?;

    let mut program = program
        .trim()
        .split(',')
        .map(|i| i.parse::<i32>().unwrap())
        .collect::<Vec<_>>();

    let stdio = io::stdin();
    let input = stdio.lock();

    let output = io::stdout();

    compute(&mut program, input, output);

    Ok(())
}

fn compute<R, W>(input: &mut [i32], mut reader: R, mut writer: W)
where
    R: BufRead,
    W: Write,
{
    let mut instruction_ptr = 0;
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
                let op_input = op_input
                    .trim()
                    .parse::<i32>()
                    .expect("Unable to parse input");
                input[input[instruction_ptr + 1] as usize] = op_input;
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
            Operation::Terminate => break,
        }
        instruction_ptr += advance;
    }
}

fn parse_operation(instruction: i32) -> Operation {
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

fn parse_parameter_mode(instruction: i32, position: u32) -> ParameterMode {
    match instruction / (10i32.pow(position + 2)) % 10 {
        0 => ParameterMode::Position,
        1 => ParameterMode::Immediate,
        _ => panic!("Unrecognized parameter mode"),
    }
}

fn value_from_parameter(input: &[i32], ptr: usize, mode: ParameterMode) -> i32 {
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
        compute(&mut program, &input[..], &mut output);
        assert_eq!(program, [3500, 9, 10, 70, 2, 3, 11, 0, 99, 30, 40, 50]);

        let mut program = [1, 0, 0, 0, 99];
        compute(&mut program, &input[..], &mut output);
        assert_eq!(program, [2, 0, 0, 0, 99]);

        let mut program = [2, 3, 0, 3, 99];
        compute(&mut program, &input[..], &mut output);
        assert_eq!(program, [2, 3, 0, 6, 99]);

        let mut program = [2, 4, 4, 5, 99, 0];
        compute(&mut program, &input[..], &mut output);
        assert_eq!(program, [2, 4, 4, 5, 99, 9801]);

        let mut program = [1, 1, 1, 4, 99, 5, 6, 0, 99];
        compute(&mut program, &input[..], &mut output);
        assert_eq!(program, [30, 1, 1, 4, 2, 5, 6, 0, 99]);

        let mut program = [1002, 4, 3, 4, 33];
        compute(&mut program, &input[..], &mut output);
        assert_eq!(program, [1002, 4, 3, 4, 99]);

        let input = b"8";
        let mut output = Vec::new();
        let mut program = [3, 9, 8, 9, 10, 9, 4, 9, 99, -1, 8];
        compute(&mut program, &input[..], &mut output);
        assert_eq!(output, b"1\n");

        let input = b"9";
        let mut output = Vec::new();
        let mut program = [3, 9, 8, 9, 10, 9, 4, 9, 99, -1, 8];
        compute(&mut program, &input[..], &mut output);
        assert_eq!(output, b"0\n");

        let input = b"8";
        let mut output = Vec::new();
        let mut program = [3, 9, 7, 9, 10, 9, 4, 9, 99, -1, 8];
        compute(&mut program, &input[..], &mut output);
        assert_eq!(output, b"0\n");

        let input = b"7";
        let mut output = Vec::new();
        let mut program = [3, 9, 7, 9, 10, 9, 4, 9, 99, -1, 8];
        compute(&mut program, &input[..], &mut output);
        assert_eq!(output, b"1\n");

        let input = b"8";
        let mut output = Vec::new();
        let mut program = [3, 3, 1108, -1, 8, 3, 4, 3, 99];
        compute(&mut program, &input[..], &mut output);
        assert_eq!(output, b"1\n");

        let input = b"9";
        let mut output = Vec::new();
        let mut program = [3, 3, 1108, -1, 8, 3, 4, 3, 99];
        compute(&mut program, &input[..], &mut output);
        assert_eq!(output, b"0\n");

        let input = b"8";
        let mut output = Vec::new();
        let mut program = [3, 3, 1107, -1, 8, 3, 4, 3, 99];
        compute(&mut program, &input[..], &mut output);
        assert_eq!(output, b"0\n");

        let input = b"7";
        let mut output = Vec::new();
        let mut program = [3, 3, 1107, -1, 8, 3, 4, 3, 99];
        compute(&mut program, &input[..], &mut output);
        assert_eq!(output, b"1\n");

        let input = b"2";
        let mut output = Vec::new();
        let mut program = [3, 12, 6, 12, 15, 1, 13, 14, 13, 4, 13, 99, -1, 0, 1, 9];
        compute(&mut program, &input[..], &mut output);
        assert_eq!(output, b"1\n");

        let input = b"0";
        let mut output = Vec::new();
        let mut program = [3, 12, 6, 12, 15, 1, 13, 14, 13, 4, 13, 99, -1, 0, 1, 9];
        compute(&mut program, &input[..], &mut output);
        assert_eq!(output, b"0\n");

        let input = b"2";
        let mut output = Vec::new();
        let mut program = [3, 3, 1105, -1, 9, 1101, 0, 0, 12, 4, 12, 99, 1];
        compute(&mut program, &input[..], &mut output);
        assert_eq!(output, b"1\n");

        let input = b"0";
        let mut output = Vec::new();
        let mut program = [3, 3, 1105, -1, 9, 1101, 0, 0, 12, 4, 12, 99, 1];
        compute(&mut program, &input[..], &mut output);
        assert_eq!(output, b"0\n");

        let program = [
            3, 21, 1008, 21, 8, 20, 1005, 20, 22, 107, 8, 21, 20, 1006, 20, 31, 1106, 0, 36, 98, 0,
            0, 1002, 21, 125, 20, 4, 20, 1105, 1, 46, 104, 999, 1105, 1, 46, 1101, 1000, 1, 20, 4,
            20, 1105, 1, 46, 98, 99,
        ];

        let input = b"7";
        let mut output = Vec::new();
        compute(&mut program.clone(), &input[..], &mut output);
        assert_eq!(output, b"999\n");

        let input = b"8";
        let mut output = Vec::new();
        compute(&mut program.clone(), &input[..], &mut output);
        assert_eq!(output, b"1000\n");

        let input = b"9";
        let mut output = Vec::new();
        compute(&mut program.clone(), &input[..], &mut output);
        assert_eq!(output, b"1001\n");
    }
}
