use std::fs::File;
use std::io::prelude::*;

fn main() {
    let mut file = File::open("input.txt").expect("Could not open file.");
    let mut image = String::new();
    file.read_to_string(&mut image)
        .expect("Could not read to string.");

    let width = 25;
    let height = 6;

    let numbers = image
        .chars()
        .map(|i| i.to_digit(10).expect("Could not parse digit."))
        .collect::<Vec<_>>();
    let (_, ones, twos) = numbers
        .chunks(width * height)
        .map(|chunk| {
            chunk
                .iter()
                .fold((0, 0, 0), |(zeros, ones, twos), i| match i {
                    0 => (zeros + 1, ones, twos),
                    1 => (zeros, ones + 1, twos),
                    2 => (zeros, ones, twos + 1),
                    i => unreachable!("Unexpected image digit: {}.", i),
                })
        })
        .min_by(|(zeroes1, _, _), (zeroes2, _, _)| zeroes1.cmp(zeroes2))
        .unwrap();

    println!("{}", ones * twos);

    let image = numbers
        .iter()
        .enumerate()
        .fold(vec![2; width * height], |mut acc, (i, n)| {
            if acc[i % (width * height)] == 2 {
                acc[i % (width * height)] = *n
            }
            acc
        });

    image.iter().enumerate().for_each(|(i, n)| {
        let c = match n {
            0 => '▓',
            1 => '░',
            _ => unreachable!(),
        };
        if i % width == 0 {
            print!("\n")
        }
        print!("{}", c);
    })
}
