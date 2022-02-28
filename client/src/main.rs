use tetrox::{DefaultField, SevenBag, Tetromino, Square};

fn main() {
    let mut bag = SevenBag::new();
    let mut playfield = DefaultField::<Tetromino>::new(10, 40, 20, &mut bag);
    playfield.try_spawn(&mut bag);

    playfield.try_shift(0, 5);

    for line in playfield.lines() {
        let squares = line.squares();
        println!(
            "{}",
            squares.iter().map(|s| match s {
                Square::Empty => "_",
                Square::Filled(_) => "#",
            }).collect::<Vec<_>>().join("")
        );
    }

    println!("client");
}
