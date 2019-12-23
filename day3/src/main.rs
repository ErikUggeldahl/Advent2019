use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

#[derive(Clone)]
struct Segment {
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
}

fn main() -> Result<(), Box<dyn Error>> {
    let file = File::open("input.txt")?;
    let mut file = BufReader::new(file);

    let mut wire1 = String::new();
    let mut wire2 = String::new();
    file.read_line(&mut wire1)?;
    file.read_line(&mut wire2)?;

    // for i in intersects1 {
    //     println!("{}", i);
    // }
    // for i in intersects2 {
    //     println!("{}", i);
    // }
    println!("{}", closest_intersection(&wire1, &wire2));

    Ok(())
}

fn closest_intersection(wire1: &str, wire2: &str) -> i32 {
    let offset = horizontal_first(wire1) == horizontal_first(wire2);

    let wire1 = wire_to_segments(&wire1).unwrap();
    let wire2 = wire_to_segments(&wire2).unwrap();

    for i in wire1.iter() {
        println!("{} {} {} {}", i.x1, i.y1, i.x2, i.y2);
    }
    println!("");
    for i in wire2.iter() {
        println!("{} {} {} {}", i.x1, i.y1, i.x2, i.y2);
    }

    let intersects1 = wire1.iter().skip(0).step_by(2).flat_map(|s1| {
        wire2
            .iter()
            .skip(if offset { 1 } else { 0 })
            .step_by(2)
            .filter_map(move |s2| segments_intersect(&s1, s2))
    });
    let intersects2 = wire1.iter().skip(1).step_by(2).flat_map(|s1| {
        wire2
            .iter()
            .skip(if offset { 0 } else { 1 })
            .step_by(2)
            .filter_map(move |s2| segments_intersect(&s1, s2))
    });
    intersects1
        .chain(intersects2)
        .inspect(|i| println!("{}", i))
        .min()
        .unwrap()
}

fn wire_to_segments(wire: &str) -> Result<Vec<Segment>, Box<dyn Error>> {
    let wire = wire.split(',');
    let mut pos_x = 0;
    let mut pos_y = 0;
    let mut segments = Vec::new();
    for i in wire {
        let amount = i.trim()[1..].parse::<i32>()?;
        match i.chars().nth(0) {
            Some('L') => {
                segments.push(Segment {
                    x1: pos_x,
                    y1: pos_y,
                    x2: pos_x - amount,
                    y2: pos_y,
                });
                pos_x -= amount
            }
            Some('R') => {
                segments.push(Segment {
                    x1: pos_x,
                    y1: pos_y,
                    x2: pos_x + amount,
                    y2: pos_y,
                });
                pos_x += amount
            }
            Some('U') => {
                segments.push(Segment {
                    x1: pos_x,
                    y1: pos_y,
                    x2: pos_x,
                    y2: pos_y + amount,
                });
                pos_y += amount
            }
            Some('D') => {
                segments.push(Segment {
                    x1: pos_x,
                    y1: pos_y,
                    x2: pos_x,
                    y2: pos_y - amount,
                });
                pos_y -= amount
            }
            _ => panic!("Unexpected input."),
        }
    }
    Ok(segments)
}

fn horizontal_first(s: &str) -> bool {
    let c = s.chars().nth(0).unwrap();
    c == 'L' || c == 'R'
}

fn segments_intersect(s1: &Segment, s2: &Segment) -> Option<i32> {
    if s1.x1 == s1.x2 {
        if contains(s1.x1, s2.x1, s2.x2) && contains(s2.y1, s1.y1, s1.y2) {
            println!(
                "{} {} {} {} and {} {} {} {} intersect",
                s1.x1, s1.y1, s1.x2, s1.y2, s2.x1, s2.y1, s2.x2, s2.y2
            );
            Some(s1.x1 + s2.y1)
        } else {
            println!(
                "{} {} {} {} and {} {} {} {} don't intersect",
                s1.x1, s1.y1, s1.x2, s1.y2, s2.x1, s2.y1, s2.x2, s2.y2
            );
            None
        }
    } else if contains(s1.y1, s2.y1, s2.y2) && contains(s2.x1, s1.x1, s1.x2) {
        println!(
            "{} {} {} {} and {} {} {} {} intersect",
            s1.x1, s1.y1, s1.x2, s1.y2, s2.x1, s2.y1, s2.x2, s2.y2
        );
        Some(s1.y1 + s2.x1)
    } else {
        println!(
            "{} {} {} {} and {} {} {} {} don't intersect",
            s1.x1, s1.y1, s1.x2, s1.y2, s2.x1, s2.y1, s2.x2, s2.y2
        );
        None
    }
}

fn contains(x: i32, x1: i32, x2: i32) -> bool {
    (x1 > x && x > x2) || (x2 < x && x < x2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_closest_intersection() {
        assert_eq!(closest_intersection("R8,U5,L5,D3", "U7,R6,D4,L4"), 6);
        // assert_eq!(closest_intersection("R75,D30,R83,U83,L12,D49,R71,U7,L72", "U62,R66,U55,R34,D71,R55,D58,R83"), 159);
        assert_eq!(closest_intersection("R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51", "U98,R91,D20,R16,D67,R40,U7,R15,U6,R7"), 135);
    }
}
