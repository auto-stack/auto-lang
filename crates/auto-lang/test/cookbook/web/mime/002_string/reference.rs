use mime::Mime;

fn main() {
    let mime: Mime = "text/html".parse().unwrap();
    println!("MIME: {}", mime);
    let is_text = mime.type_() == mime::TEXT;
    println!("Is text: {}", is_text);
}
