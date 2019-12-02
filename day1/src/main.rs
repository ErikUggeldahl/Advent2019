use std::fs::File;
use std::io::{self, BufRead};

fn main() -> Result<(), io::Error> {
    let file = File::open("input.txt")?;

    let total_fuel = io::BufReader::new(file)
        .lines()
        .filter_map(Result::ok)
        .fold(0, |acc, mass| {
            acc + recursive_fuel(mass.parse::<i32>().unwrap())
        });

    println!("{}", total_fuel);
    Ok(())
}

fn simple_fuel(mass: i32) -> i32 {
    mass / 3 - 2
}

fn recursive_fuel(mut mass: i32) -> i32 {
    let mut sum = 0;
    loop {
        let fuel = simple_fuel(mass);
        if fuel <= 0 {
            break;
        }
        sum += fuel;
        mass = fuel;
    }
    sum
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_fuel() {
        assert_eq!(simple_fuel(12), 2);
        assert_eq!(simple_fuel(14), 2);
        assert_eq!(simple_fuel(1969), 654);
        assert_eq!(simple_fuel(100_756), 33583);
    }

    #[test]
    fn test_recursive_fuel() {
        assert_eq!(recursive_fuel(14), 2);
        assert_eq!(recursive_fuel(1969), 966);
        assert_eq!(recursive_fuel(100_756), 50346);
    }
}
