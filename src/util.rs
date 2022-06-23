pub fn qrcode2str(data: &[u8]) -> String {
    let image = image::load_from_memory(data).unwrap().into_luma8();
    let mut s = String::new();
    for j in 0..22 {
        for i in 0..44 {
            let top = image.get_pixel(3 * i, 6 * j).0[0];
            let bottom = image.get_pixel(3 * i, 6 * j + 3).0[0];
            match (top, bottom) {
                (0, 0) => s.push('█'),
                (_, 0) => s.push('▄'),
                (0, _) => s.push('▀'),
                (_, _) => s.push(' '),
            }
        }
        s.push('\n');
    }
    s
}
