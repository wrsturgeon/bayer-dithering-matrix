use rayon::iter::ParallelIterator;

fn main() {
    const BAYER: [[u8; 16]; 16] = bayer_dithering_matrix::matrix();

    let mut im = image::ImageReader::open("examples/windows-xp-bliss.png")
        .unwrap()
        .decode()
        .unwrap()
        .into_luma8();

    let () = im
        .par_enumerate_pixels_mut()
        .for_each(|(i, j, &mut image::Luma([ref mut lum]))| {
            let threshold = unsafe {
                BAYER
                    .get_unchecked((i & 15) as usize)
                    .get_unchecked((j & 15) as usize)
            };
            if *lum > *threshold {
                *lum = u8::MAX
            } else {
                *lum = u8::MIN
            }
        });

    let () = im.save("examples/windows-xp-bliss-processed.png").unwrap();
}
