use std::fs::File;
use std::io::prelude::*;
use std::io::{self};

use intcode::{ExitStatus, Intcode};

use permute::permutations_of;

fn main() -> Result<(), io::Error> {
    let mut file = File::open("input.txt")?;
    let mut program = String::new();
    file.read_to_string(&mut program)?;

    let program = program
        .trim()
        .split(',')
        .map(|i| i.parse::<i64>().unwrap())
        .collect::<Vec<_>>();

    let computer = Intcode::new(program);

    let max = permutations_of(&[0, 1, 2, 3, 4])
        .map(permutation_to_array)
        .map(|permutation| amplifier_sequence(computer.clone(), &permutation))
        .max()
        .unwrap();

    println!("{}", max);

    let max = permutations_of(&[5, 6, 7, 8, 9])
        .map(permutation_to_array)
        .map(|permutation| amplifier_sequence_loop(computer.clone(), &permutation))
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

fn amplifier_sequence(computer: Intcode, phase_settings: &[u32; 5]) -> i64 {
    let mut input_signal = String::from("0");

    for phase in phase_settings.iter() {
        let mut computer = computer.clone();

        let input = format!("{}\n{}", phase.to_string(), input_signal);
        let mut output = Vec::new();

        computer.compute(input.as_bytes(), &mut output);

        input_signal = String::from_utf8(output).unwrap();
    }

    input_signal
        .trim()
        .parse()
        .expect("Unexpected final amplifier value")
}

fn amplifier_sequence_loop(computer: Intcode, phase_settings: &[u32; 5]) -> i64 {
    let mut computers = (0..5).map(|_| computer.clone()).collect::<Vec<_>>();
    let mut input_signal = String::from("0");
    let mut exit_status: ExitStatus = ExitStatus::AwaitingInput;
    let mut first = true;

    while exit_status != ExitStatus::Terminated {
        for (i, phase) in phase_settings.iter().enumerate() {
            let input = if first {
                format!("{}\n{}", phase.to_string(), input_signal)
            } else {
                input_signal
            };
            let mut output = Vec::new();

            exit_status = computers[i].compute(input.as_bytes(), &mut output);

            input_signal = String::from_utf8(output).unwrap();
        }
        first = false;
    }

    input_signal
        .trim()
        .parse()
        .expect("Unexpected final amplifier value")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_amplifier_sequence() {
        let computer = Intcode::new(
            [
                3, 15, 3, 16, 1002, 16, 10, 16, 1, 16, 15, 15, 4, 15, 99, 0, 0,
            ]
            .to_vec(),
        );
        assert_eq!(amplifier_sequence(computer, &[4, 3, 2, 1, 0]), 43210);

        let computer = Intcode::new(
            [
                3, 23, 3, 24, 1002, 24, 10, 24, 1002, 23, -1, 23, 101, 5, 23, 23, 1, 24, 23, 23, 4,
                23, 99, 0, 0,
            ]
            .to_vec(),
        );
        assert_eq!(amplifier_sequence(computer, &[0, 1, 2, 3, 4]), 54321);

        let computer = Intcode::new(
            [
                3, 31, 3, 32, 1002, 32, 10, 32, 1001, 31, -2, 31, 1007, 31, 0, 33, 1002, 33, 7, 33,
                1, 33, 31, 31, 1, 32, 31, 31, 4, 31, 99, 0, 0, 0,
            ]
            .to_vec(),
        );
        assert_eq!(amplifier_sequence(computer, &[1, 0, 4, 3, 2]), 65210);
    }

    #[test]
    fn test_amplifier_sequence_loop() {
        let computer = Intcode::new(
            [
                3, 26, 1001, 26, -4, 26, 3, 27, 1002, 27, 2, 27, 1, 27, 26, 27, 4, 27, 1001, 28,
                -1, 28, 1005, 28, 6, 99, 0, 0, 5,
            ]
            .to_vec(),
        );
        assert_eq!(
            amplifier_sequence_loop(computer, &[9, 8, 7, 6, 5]),
            139629729
        );

        let computer = Intcode::new(
            [
                3, 52, 1001, 52, -5, 52, 3, 53, 1, 52, 56, 54, 1007, 54, 5, 55, 1005, 55, 26, 1001,
                54, -5, 54, 1105, 1, 12, 1, 53, 54, 53, 1008, 54, 0, 55, 1001, 55, 1, 55, 2, 53,
                55, 53, 4, 53, 1001, 56, -1, 56, 1005, 56, 6, 99, 0, 0, 0, 0, 10,
            ]
            .to_vec(),
        );
        assert_eq!(amplifier_sequence_loop(computer, &[9, 7, 8, 5, 6]), 18216);
    }
}
