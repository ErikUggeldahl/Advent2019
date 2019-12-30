use std::io::prelude::*;

#[derive(Clone)]
pub struct Intcode {
    pub program: Vec<i64>,
    instruction_ptr: usize,
    relative_base: i64,
}

impl Intcode {
    pub fn new(mut program: Vec<i64>) -> Self {
        program.extend_from_slice(&vec![0; 2048 - program.len()]);
        Intcode {
            program,
            instruction_ptr: 0,
            relative_base: 0,
        }
    }
}

enum ParameterMode {
    Position,
    Immediate,
    Relative,
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
    ChangeRelativeBase(ParameterMode),
    Terminate,
}

impl Operation {
    fn advance(&self) -> usize {
        match self {
            Self::Addition(_, _, _) => 4,
            Self::Multiplication(_, _, _) => 4,
            Self::Input(_) => 2,
            Self::Output(_) => 2,
            Self::JumpTrue(_, _) => 3,
            Self::JumpFalse(_, _) => 3,
            Self::Less(_, _, _) => 4,
            Self::Equal(_, _, _) => 4,
            Self::ChangeRelativeBase(_) => 2,
            Self::Terminate => 0,
        }
    }
}

#[derive(PartialEq)]
pub enum ExitStatus {
    AwaitingInput,
    Terminated,
}

impl Intcode {
    pub fn compute<R, W>(&mut self, mut reader: R, mut writer: W) -> ExitStatus
    where
        R: BufRead,
        W: Write,
    {
        loop {
            let operation = Self::parse_operation(self.program[self.instruction_ptr]);
            let mut advance = operation.advance();
            match operation {
                Operation::Addition(p1, p2, p3) => {
                    self.write(
                        3,
                        self.value_from_parameter(1, p1) + self.value_from_parameter(2, p2),
                        p3,
                    );
                }
                Operation::Multiplication(p1, p2, p3) => {
                    self.write(
                        3,
                        self.value_from_parameter(1, p1) * self.value_from_parameter(2, p2),
                        p3,
                    );
                }
                Operation::Input(p1) => {
                    let mut op_input = String::new();
                    reader.read_line(&mut op_input).expect("Unable to read");
                    let op_input = op_input.trim().parse::<i64>();
                    match op_input {
                        Ok(i) => self.write(1, i, p1),
                        Err(_) => return ExitStatus::AwaitingInput,
                    }
                }
                Operation::Output(p1) => {
                    writeln!(&mut writer, "{}", self.value_from_parameter(1, p1))
                        .expect("Unable to write");
                }
                Operation::JumpTrue(p1, p2) => {
                    if self.value_from_parameter(1, p1) != 0 {
                        advance = 0;
                        self.instruction_ptr = self.value_from_parameter(2, p2) as usize;
                    }
                }
                Operation::JumpFalse(p1, p2) => {
                    if self.value_from_parameter(1, p1) == 0 {
                        advance = 0;
                        self.instruction_ptr = self.value_from_parameter(2, p2) as usize;
                    }
                }
                Operation::Less(p1, p2, p3) => {
                    self.write(
                        3,
                        match self.value_from_parameter(1, p1) < self.value_from_parameter(2, p2) {
                            true => 1,
                            false => 0,
                        },
                        p3,
                    );
                }
                Operation::Equal(p1, p2, p3) => {
                    self.write(
                        3,
                        match self.value_from_parameter(1, p1) == self.value_from_parameter(2, p2) {
                            true => 1,
                            false => 0,
                        },
                        p3,
                    );
                }
                Operation::ChangeRelativeBase(p1) => {
                    self.relative_base += self.value_from_parameter(1, p1);
                }
                Operation::Terminate => return ExitStatus::Terminated,
            }
            self.instruction_ptr += advance;
        }
    }

    fn write(&mut self, offset: usize, value: i64, mode: ParameterMode) {
        let address = match mode {
            ParameterMode::Position => self.program[self.instruction_ptr + offset] as usize,
            ParameterMode::Relative => {
                (self.program[self.instruction_ptr + offset] + self.relative_base) as usize
            }
            _ => unreachable!("Cannot write with immediate mode."),
        };
        self.program[address] = value;
    }

    fn parse_operation(instruction: i64) -> Operation {
        let op_code = instruction % 100;
        match op_code {
            1 => Operation::Addition(
                Self::parse_parameter_mode(instruction, 0),
                Self::parse_parameter_mode(instruction, 1),
                Self::parse_parameter_mode(instruction, 2),
            ),
            2 => Operation::Multiplication(
                Self::parse_parameter_mode(instruction, 0),
                Self::parse_parameter_mode(instruction, 1),
                Self::parse_parameter_mode(instruction, 2),
            ),
            3 => Operation::Input(Self::parse_parameter_mode(instruction, 0)),
            4 => Operation::Output(Self::parse_parameter_mode(instruction, 0)),
            5 => Operation::JumpTrue(
                Self::parse_parameter_mode(instruction, 0),
                Self::parse_parameter_mode(instruction, 1),
            ),
            6 => Operation::JumpFalse(
                Self::parse_parameter_mode(instruction, 0),
                Self::parse_parameter_mode(instruction, 1),
            ),
            7 => Operation::Less(
                Self::parse_parameter_mode(instruction, 0),
                Self::parse_parameter_mode(instruction, 1),
                Self::parse_parameter_mode(instruction, 2),
            ),
            8 => Operation::Equal(
                Self::parse_parameter_mode(instruction, 0),
                Self::parse_parameter_mode(instruction, 1),
                Self::parse_parameter_mode(instruction, 2),
            ),
            9 => Operation::ChangeRelativeBase(Self::parse_parameter_mode(instruction, 0)),
            99 => Operation::Terminate,
            c => panic!("Unrecognized instruction: {}", c),
        }
    }

    fn parse_parameter_mode(instruction: i64, position: u32) -> ParameterMode {
        match instruction / (10i64.pow(position + 2)) % 10 {
            0 => ParameterMode::Position,
            1 => ParameterMode::Immediate,
            2 => ParameterMode::Relative,
            p => panic!("Unrecognized parameter mode: {}", p),
        }
    }

    fn value_from_parameter(&self, offset: usize, mode: ParameterMode) -> i64 {
        match mode {
            ParameterMode::Position => {
                self.program[self.program[self.instruction_ptr + offset] as usize]
            }
            ParameterMode::Immediate => self.program[self.instruction_ptr + offset],
            ParameterMode::Relative => {
                self.program
                    [(self.program[self.instruction_ptr + offset] + self.relative_base) as usize]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::{empty, sink};

    use super::*;

    #[test]
    fn test_simple() {
        let mut computer = Intcode::new([1, 9, 10, 3, 2, 3, 11, 0, 99, 30, 40, 50].to_vec());
        computer.compute(empty(), sink());
        assert_eq!(
            computer.program,
            [3500, 9, 10, 70, 2, 3, 11, 0, 99, 30, 40, 50].to_vec()
        );

        let mut computer = Intcode::new([1, 0, 0, 0, 99].to_vec());
        computer.compute(empty(), sink());
        assert_eq!(computer.program, [2, 0, 0, 0, 99]);

        let mut computer = Intcode::new([2, 3, 0, 3, 99].to_vec());
        computer.compute(empty(), sink());
        assert_eq!(computer.program, [2, 3, 0, 6, 99]);

        let mut computer = Intcode::new([2, 4, 4, 5, 99, 0].to_vec());
        computer.compute(empty(), sink());
        assert_eq!(computer.program, [2, 4, 4, 5, 99, 9801]);

        let mut computer = Intcode::new([1, 1, 1, 4, 99, 5, 6, 0, 99].to_vec());
        computer.compute(empty(), sink());
        assert_eq!(computer.program, [30, 1, 1, 4, 2, 5, 6, 0, 99]);
    }

    #[test]
    fn test_operations() {
        let mut computer = Intcode::new([1002, 4, 3, 4, 33].to_vec());
        computer.compute(empty(), sink());
        assert_eq!(computer.program, [1002, 4, 3, 4, 99]);

        let input = b"8";
        let mut output = Vec::new();
        let mut computer = Intcode::new([3, 9, 8, 9, 10, 9, 4, 9, 99, -1, 8].to_vec());
        computer.compute(&input[..], &mut output);
        assert_eq!(output, b"1\n");

        let input = b"9";
        let mut output = Vec::new();
        let mut computer = Intcode::new([3, 9, 8, 9, 10, 9, 4, 9, 99, -1, 8].to_vec());
        computer.compute(&input[..], &mut output);
        assert_eq!(output, b"0\n");

        let input = b"8";
        let mut output = Vec::new();
        let mut computer = Intcode::new([3, 9, 7, 9, 10, 9, 4, 9, 99, -1, 8].to_vec());
        computer.compute(&input[..], &mut output);
        assert_eq!(output, b"0\n");

        let input = b"7";
        let mut output = Vec::new();
        let mut computer = Intcode::new([3, 9, 7, 9, 10, 9, 4, 9, 99, -1, 8].to_vec());
        computer.compute(&input[..], &mut output);
        assert_eq!(output, b"1\n");

        let input = b"8";
        let mut output = Vec::new();
        let mut computer = Intcode::new([3, 3, 1108, -1, 8, 3, 4, 3, 99].to_vec());
        computer.compute(&input[..], &mut output);
        assert_eq!(output, b"1\n");

        let input = b"9";
        let mut output = Vec::new();
        let mut computer = Intcode::new([3, 3, 1108, -1, 8, 3, 4, 3, 99].to_vec());
        computer.compute(&input[..], &mut output);
        assert_eq!(output, b"0\n");

        let input = b"8";
        let mut output = Vec::new();
        let mut computer = Intcode::new([3, 3, 1107, -1, 8, 3, 4, 3, 99].to_vec());
        computer.compute(&input[..], &mut output);
        assert_eq!(output, b"0\n");

        let input = b"7";
        let mut output = Vec::new();
        let mut computer = Intcode::new([3, 3, 1107, -1, 8, 3, 4, 3, 99].to_vec());
        computer.compute(&input[..], &mut output);
        assert_eq!(output, b"1\n");

        let input = b"2";
        let mut output = Vec::new();
        let mut computer =
            Intcode::new([3, 12, 6, 12, 15, 1, 13, 14, 13, 4, 13, 99, -1, 0, 1, 9].to_vec());
        computer.compute(&input[..], &mut output);
        assert_eq!(output, b"1\n");

        let input = b"0";
        let mut output = Vec::new();
        let mut computer =
            Intcode::new([3, 12, 6, 12, 15, 1, 13, 14, 13, 4, 13, 99, -1, 0, 1, 9].to_vec());
        computer.compute(&input[..], &mut output);
        assert_eq!(output, b"0\n");

        let input = b"2";
        let mut output = Vec::new();
        let mut computer = Intcode::new([3, 3, 1105, -1, 9, 1101, 0, 0, 12, 4, 12, 99, 1].to_vec());
        computer.compute(&input[..], &mut output);
        assert_eq!(output, b"1\n");

        let input = b"0";
        let mut output = Vec::new();
        let mut computer = Intcode::new([3, 3, 1105, -1, 9, 1101, 0, 0, 12, 4, 12, 99, 1].to_vec());
        computer.compute(&input[..], &mut output);
        assert_eq!(output, b"0\n");

        let computer = Intcode::new(
            [
                3, 21, 1008, 21, 8, 20, 1005, 20, 22, 107, 8, 21, 20, 1006, 20, 31, 1106, 0, 36,
                98, 0, 0, 1002, 21, 125, 20, 4, 20, 1105, 1, 46, 104, 999, 1105, 1, 46, 1101, 1000,
                1, 20, 4, 20, 1105, 1, 46, 98, 99,
            ]
            .to_vec(),
        );

        let input = b"7";
        let mut output = Vec::new();
        let mut comp = computer.clone();
        comp.compute(&input[..], &mut output);
        assert_eq!(output, b"999\n");

        let input = b"8";
        let mut output = Vec::new();
        let mut comp = computer.clone();
        comp.compute(&input[..], &mut output);
        assert_eq!(output, b"1000\n");

        let input = b"9";
        let mut output = Vec::new();
        let mut comp = computer.clone();
        comp.compute(&input[..], &mut output);
        assert_eq!(output, b"1001\n");
    }

    #[test]
    fn test_relative_mode() {
        let mut output = Vec::new();
        let mut comp = Intcode::new(
            [
                109, 1, 204, -1, 1001, 100, 1, 100, 1008, 100, 16, 101, 1006, 101, 0, 99,
            ]
            .to_vec(),
        );
        comp.compute(empty(), &mut output);
        assert_eq!(
            String::from_utf8(output).unwrap(),
            "109\n1\n204\n-1\n1001\n100\n1\n100\n1008\n100\n16\n101\n1006\n101\n0\n99\n"
        );

        let mut output = Vec::new();
        let mut comp = Intcode::new([1102, 34915192, 34915192, 7, 4, 7, 99, 0].to_vec());
        comp.compute(empty(), &mut output);
        let result = String::from_utf8(output)
            .unwrap()
            .trim()
            .parse::<u64>()
            .unwrap();
        assert!(result / 1_000_000_000_000_000 > 0);

        let mut output = Vec::new();
        let mut comp = Intcode::new([104, 1125899906842624, 99].to_vec());
        comp.compute(empty(), &mut output);
        let result = String::from_utf8(output)
            .unwrap()
            .trim()
            .parse::<u64>()
            .unwrap();
        assert_eq!(result, 1125899906842624);
    }
}
