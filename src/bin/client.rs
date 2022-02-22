use tetrox::{DefaultField, Tetromino, SevenBag, Field};

fn main() {
    let mut bag = SevenBag::new();
    let mut playfield = DefaultField::<Tetromino>::new(10, 40, 20);
    playfield.spawn_piece(&mut bag);
    
    println!("client");
}
