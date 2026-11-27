use std::path::Path;

use fovea_render::{DecodedFrame, RenderError, extract_video_frame};

#[test]
fn missing_video_returns_not_found() {
    let err = extract_video_frame(Path::new("/tmp/definitely-missing-fovea.mov"), 0, 1280)
        .expect_err("must fail");
    assert!(matches!(err, RenderError::NotFound(_)));
}

#[test]
fn decoded_frame_validates_bgra_byte_length() {
    let frame = DecodedFrame {
        width: 4,
        height: 2,
        bgra: vec![0_u8; 32],
    };
    assert!(frame.byte_len_valid());
}
