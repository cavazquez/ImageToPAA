//! Semantic PAA tags: CGVA, CXAM, GALF (order without SFFO).

use crate::options::PaFormat;
use image::RgbaImage;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag {
    pub name: [u8; 4],
    pub payload: Vec<u8>,
}

impl Tag {
    pub fn new(name: &[u8; 4], payload: Vec<u8>) -> Self {
        Self {
            name: *name,
            payload,
        }
    }
}

/// Average and max colour stats over the base image.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColourStats {
    pub avg_bgra: [u8; 4],
    pub max_bgra: [u8; 4],
    pub has_alpha: bool,
}

pub fn colour_stats(image: &RgbaImage) -> ColourStats {
    let (w, h) = image.dimensions();
    let mut sum_r: u64 = 0;
    let mut sum_g: u64 = 0;
    let mut sum_b: u64 = 0;
    let mut sum_a: u64 = 0;
    let mut max_r: u8 = 0;
    let mut max_g: u8 = 0;
    let mut max_b: u8 = 0;
    let mut max_a: u8 = 0;
    let mut has_alpha = false;

    for px in image.pixels() {
        let [r, g, b, a] = px.0;
        sum_r += u64::from(r);
        sum_g += u64::from(g);
        sum_b += u64::from(b);
        sum_a += u64::from(a);
        max_r = max_r.max(r);
        max_g = max_g.max(g);
        max_b = max_b.max(b);
        max_a = max_a.max(a);
        if a < 255 {
            has_alpha = true;
        }
    }

    let n = u64::from(w) * u64::from(h);
    let avg = match (
        sum_b.checked_div(n),
        sum_g.checked_div(n),
        sum_r.checked_div(n),
        sum_a.checked_div(n),
    ) {
        (Some(b), Some(g), Some(r), Some(a)) => [b as u8, g as u8, r as u8, a as u8],
        _ => [0, 0, 0, 0],
    };

    ColourStats {
        avg_bgra: avg,
        max_bgra: [max_b, max_g, max_r, max_a],
        has_alpha,
    }
}

/// Build semantic tags (CGVA, CXAM, optional GALF) for a resolved DXT format.
/// CXAM is forced to FF FF FF FF for DXT per the MVP profile.
pub fn build_semantic_tags(image: &RgbaImage, format: PaFormat) -> Vec<Tag> {
    debug_assert!(matches!(format, PaFormat::Dxt1 | PaFormat::Dxt5));
    let stats = colour_stats(image);
    let mut tags = Vec::with_capacity(3);
    tags.push(Tag::new(b"CGVA", stats.avg_bgra.to_vec()));
    tags.push(Tag::new(b"CXAM", vec![0xFF, 0xFF, 0xFF, 0xFF]));
    if format == PaFormat::Dxt5 {
        tags.push(Tag::new(b"GALF", vec![1, 0, 0, 0]));
    }
    tags
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    #[test]
    fn avg_bgra_order_differs_from_rgba() {
        // Pure red → RGBA avg (255,0,0,255) but CGVA BGRA (0,0,255,255)
        let img = RgbaImage::from_pixel(4, 4, Rgba([255, 0, 0, 255]));
        let s = colour_stats(&img);
        assert_eq!(s.avg_bgra, [0, 0, 255, 255]);
        assert_ne!(s.avg_bgra, [255, 0, 0, 255]);
    }

    #[test]
    fn wide_accumulators_do_not_overflow_conceptually() {
        // 4096*4096*255 fits in u64.
        let max = 4096u64 * 4096 * 255;
        assert!(max < u64::MAX / 2);
    }

    #[test]
    fn tag_order_dxt1_no_galf() {
        let img = RgbaImage::from_pixel(4, 4, Rgba([10, 20, 30, 255]));
        let tags = build_semantic_tags(&img, PaFormat::Dxt1);
        assert_eq!(tags.len(), 2);
        assert_eq!(&tags[0].name, b"CGVA");
        assert_eq!(&tags[1].name, b"CXAM");
        assert_eq!(tags[1].payload, [0xFF; 4]);
    }

    #[test]
    fn tag_order_dxt5_with_galf() {
        let img = RgbaImage::from_pixel(4, 4, Rgba([1, 2, 3, 128]));
        let tags = build_semantic_tags(&img, PaFormat::Dxt5);
        assert_eq!(tags.len(), 3);
        assert_eq!(&tags[0].name, b"CGVA");
        assert_eq!(&tags[1].name, b"CXAM");
        assert_eq!(&tags[2].name, b"GALF");
        assert_eq!(tags[2].payload, [1, 0, 0, 0]);
    }
}
