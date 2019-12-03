use std::fs::File;
use std::io::{self};
use std::io::prelude::*;

fn main() -> Result<(), io::Error> {
    let mut file = File::open("input.txt")?;
    let mut program = String::new();
    file.read_to_string(&mut program)?;

    let program = program.split(',').map(|i| i.parse::<i32>().unwrap()).collect::<Vec<_>>();

    let mut program1 = program.clone();
    program1[1] = 12;
    program1[2] = 2;

    compute(&mut program1);

    println!("{}", program1[0]);

    for i in 0..99 {
        for j in 0..99 {
            let mut program2 = program.clone();
            program2[1] = i;
            program2[2] = j;
            compute(&mut program2);


            if program2[0] == 19_690_720 {
                println!("{}", i * 100 + j);
                return Ok(());
            }
        }
    }

    Ok(())
}

fn compute(input: &mut [i32]) {
    let mut instruction_ptr = 0;
    loop {
        match input[instruction_ptr] {
            1 => input[input[instruction_ptr + 3] as usize] = input[input[instruction_ptr + 1] as usize] + input[input[instruction_ptr + 2] as usize],
            2 => input[input[instruction_ptr + 3] as usize] = input[input[instruction_ptr + 1] as usize] * input[input[instruction_ptr + 2] as usize],
            99 => break,
            _ => panic!("Unrecognized instruction"),
        }
        instruction_ptr += 4;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_computer() {
        let mut program = [1,9,10,3,2,3,11,0,99,30,40,50]; 
        compute(&mut program);
        assert_eq!(program, [3500,9,10,70,2,3,11,0,99,30,40,50]);

        let mut program = [1,0,0,0,99];
        compute(&mut program);
        assert_eq!(program, [2,0,0,0,99]);

        let mut program = [2,3,0,3,99];
        compute(&mut program);
        assert_eq!(program, [2,3,0,6,99]);

        let mut program = [2,4,4,5,99,0];
        compute(&mut program);
        assert_eq!(program, [2,4,4,5,99,9801]);

        let mut program = [1,1,1,4,99,5,6,0,99];
        compute(&mut program);
        assert_eq!(program, [30,1,1,4,2,5,6,0,99]);
    }
}
