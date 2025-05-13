#[test]
fn test_ctor() {
    let gf = jerasure_rs::galois::GaloisField::try_from_code_word(jerasure_rs::CodeWord::W8);
    assert!(gf.is_some());
    let gf = jerasure_rs::galois::GaloisField::try_from_code_word(jerasure_rs::CodeWord::W16);
    assert!(gf.is_some());
    let gf = jerasure_rs::galois::GaloisField::try_from_code_word(jerasure_rs::CodeWord::W32);
    assert!(gf.is_some());
    let gf = jerasure_rs::galois::GaloisField::try_from_code_word(jerasure_rs::CodeWord::Other(0));
    assert!(gf.is_none());
    let gf = jerasure_rs::galois::GaloisField::try_from_code_word(jerasure_rs::CodeWord::Other(33));
    assert!(gf.is_none());
}

#[test]
fn test_w8_region_mult() {
    let gf =
        jerasure_rs::galois::GaloisField::try_from_code_word(jerasure_rs::CodeWord::W8).unwrap();
    let src = [
        0xc4, 0xfa, 0x87, 0xee, 0x9a, 0x57, 0xcd, 0x56, 0xe2, 0xc2, 0xea, 0x11, 0xcc, 0x59, 0x84,
        0x26,
    ];
    let expect_out = [
        0x90, 0x27, 0xba, 0xfe, 0xae, 0xf3, 0x5d, 0x1d, 0x42, 0xce, 0x61, 0xa8, 0xb3, 0x8e, 0x95,
        0xd2,
    ];
    let mut out = [0_u8; 16];
    let src_in = src.clone();
    gf.region_multiply(src_in.as_slice(), 238, 0, &mut out)
        .unwrap();
    assert_eq!(expect_out, out);
    assert_eq!(src, src_in);

    let src = [
        0xe4, 0x6e, 0xc4, 0x84, 0xc8, 0xc1, 0x13, 0x04, 0x68, 0x76, 0x01, 0x09, 0x12, 0x7d, 0x82,
        0xaa,
    ];
    let expect_out = [
        0x3a, 0x35, 0x25, 0x1b, 0x8c, 0x92, 0xec, 0x67, 0xef, 0x7a, 0xd0, 0x1e, 0x3c, 0xd9, 0xc1,
        0x10,
    ];
    let mut out = [0_u8; 16];
    let src_in = src.clone();
    gf.region_multiply(src_in.as_slice(), 208, 80, &mut out)
        .unwrap();
    assert_eq!(expect_out, out);
    assert_eq!(src, src_in);
}

#[test]
fn test_w8_region_xor() {
    let gf =
        jerasure_rs::galois::GaloisField::try_from_code_word(jerasure_rs::CodeWord::W8).unwrap();
    let src_a = [0xc4, 0xfa, 0x87, 0xee, 0x9a, 0x57, 0xcd, 0x56];
    let src_b = [0x9a, 0x57, 0xcd, 0x56, 0xc4, 0xfa, 0x87, 0xee];
    let expect_out = [0x5e, 0xad, 0x4a, 0xb8, 0x5e, 0xad, 0x4a, 0xb8];
    let mut out = [0_u8; 8];
    let src_a_in = src_a.clone();
    let src_b_in = src_b.clone();
    gf.region_add(src_a.as_slice(), src_b.as_slice(), &mut out)
        .unwrap();
    assert_eq!(expect_out, out);
    assert_eq!(src_a, src_a_in);
    assert_eq!(src_b, src_b_in);

    let mut buf = src_a.clone();
    let acc = src_b.clone();
    gf.region_acc(&mut buf, &acc).unwrap();
    assert_eq!(buf, expect_out);
    assert_eq!(acc, src_b);
}
