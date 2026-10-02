use crate::develop::domain::zone::PersonPart;

use super::photo_view::PhotoRegion;

/// The class of the model of body parts that is the face.
pub const BODY_FACE_CLASS: usize = 11;
/// What the model of face parts finds in a face that is not skin: glasses,
/// eyes, eyebrows, the inside of the mouth, lips.
pub const FACE_CLASSES_NOT_SKIN: [usize; 8] = [3, 4, 5, 6, 7, 10, 11, 12];
// A face is looked at with what surrounds it, as the model was trained.
const FACE_MARGIN: f32 = 1.6;

/// The classes of the model of body parts that make `part`: face, arms and
/// legs are skin; hat, shoes and scarf are clothes.
pub fn body_classes(part: PersonPart) -> &'static [usize] {
    match part {
        PersonPart::Skin => &[11, 12, 13, 14, 15],
        PersonPart::Hair => &[2],
        PersonPart::Clothes => &[1, 4, 5, 6, 7, 8, 9, 10, 17],
        PersonPart::Eyes | PersonPart::Lips => &[],
    }
}

/// The classes of the model of face parts that make `part`.
pub fn face_classes(part: PersonPart) -> &'static [usize] {
    match part {
        PersonPart::Eyes => &[4, 5],
        PersonPart::Lips => &[11, 12],
        PersonPart::Skin | PersonPart::Hair | PersonPart::Clothes => &[],
    }
}

/// Whether masking `part` needs the faces to be looked at: skin leaves out
/// the eyes and the lips.
pub fn needs_face(part: PersonPart) -> bool {
    matches!(part, PersonPart::Eyes | PersonPart::Lips | PersonPart::Skin)
}

/// What the model of face parts is shown of a photo of `photo_size` pixels
/// for a `face`: a square of pixels around it, cut where it leaves the photo.
pub fn face_surroundings(face: &PhotoRegion, photo_size: [u32; 2]) -> PhotoRegion {
    let in_pixels = [0, 1].map(|axis| face.size[axis] * photo_size[axis] as f32);
    let side = in_pixels[0].max(in_pixels[1]) * FACE_MARGIN;
    let size = [0, 1].map(|axis| side / photo_size[axis] as f32);
    PhotoRegion::around(face.centre(), size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn face_is_seen_in_a_square_of_pixels_with_its_surroundings() {
        let face = PhotoRegion {
            min: [0.45, 0.4],
            size: [0.1, 0.1],
        };

        let seen = face_surroundings(&face, [2000, 1000]);

        assert!((seen.size[0] - 0.16).abs() < 1e-6 && (seen.size[1] - 0.32).abs() < 1e-6);
        assert!((seen.centre()[0] - 0.5).abs() < 1e-6 && (seen.centre()[1] - 0.45).abs() < 1e-6);
    }
}
