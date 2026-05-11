use unicode_segmentation::UnicodeSegmentation;

fn main() {
    let text = "hello world";
    let graphemes: Vec<&str> = text.graphemes(true).collect();
    println!("Graphemes: {:?}", graphemes);
    let reversed: String = text.graphemes(true).rev().collect();
    println!("Reversed: {}", reversed);
}
