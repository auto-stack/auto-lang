use mime_guess;

fn main() {
    let mime = mime_guess::from_path("document.pdf").first_or_octet_stream();
    println!("MIME type: {}", mime);
    let img_mime = mime_guess::from_path("image.png").first_or_octet_stream();
    println!("Image MIME: {}", img_mime);
}
