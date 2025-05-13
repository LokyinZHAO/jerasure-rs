use std::num::NonZeroI32;

use jerasure_rs::erasure::CodingMethod;
use rand::seq::SliceRandom;

fn make_rand_blk(n: usize, blk_size: usize) -> Vec<Vec<u8>> {
    (0..n)
        .map(|_| rand::random_iter().take(blk_size).collect::<Vec<u8>>())
        .collect()
}

fn make_zero_blk(n: usize, blk_size: usize) -> Vec<Vec<u8>> {
    (0..n).map(|_| vec![0_u8; blk_size]).collect()
}

const BLK_SIZE: usize = 1 << 20; // 1MB

#[test]
fn reed_sol() -> Result<(), Box<dyn std::error::Error>> {
    let k = 4;
    let m = 2;
    let method = CodingMethod::ReedSolVand;

    // test matrix
    test_matrix(k, m, method)?;

    // the rest tech not supported
    let method = CodingMethod::ReedSolVand;
    use jerasure_rs::erasure::Technique;
    for tech in [
        Technique::BitMatrix,
        Technique::Schedule,
        Technique::ScheduleCache,
    ] {
        let ec = jerasure_rs::erasure::ErasureCodeBuilder::new()
            .coding_method(method)
            .k(NonZeroI32::new(4).unwrap())
            .m(NonZeroI32::new(2).unwrap())
            .tech(tech)
            .build();
        assert!(matches!(ec, Err(jerasure_rs::Error::NotSupported(_))));
    }

    // fail test
    fail_test(k, m, method);
    let ec = jerasure_rs::erasure::ErasureCodeBuilder::new()
        .coding_method(method)
        .k(NonZeroI32::new(k).unwrap())
        .m(NonZeroI32::new(m).unwrap())
        .w(jerasure_rs::CodeWord::Other(12))
        .tech(jerasure_rs::erasure::Technique::Matrix)
        .build();
    assert!(matches!(ec, Err(jerasure_rs::Error::NotSupported(_))));

    Ok(())
}

#[test]
fn cauchy() -> Result<(), Box<dyn std::error::Error>> {
    let k = 4;
    let m = 2;
    let method = CodingMethod::Cauchy;
    // test matrix
    test_matrix(k, m, method)?;
    // bitmatrix
    test_bitmatrix(k, m, method)?;
    // schedule
    test_sechdule(k, m, method)?;

    // fail test
    fail_test(k, m, method);

    Ok(())
}

fn test_matrix(k: i32, m: i32, method: CodingMethod) -> Result<(), Box<dyn std::error::Error>> {
    let ec = jerasure_rs::erasure::ErasureCodeBuilder::new()
        .coding_method(method)
        .k(NonZeroI32::new(k).unwrap())
        .m(NonZeroI32::new(m).unwrap())
        .tech(jerasure_rs::erasure::Technique::Matrix)
        .build()?;
    general_test(ec)
}

fn test_bitmatrix(k: i32, m: i32, method: CodingMethod) -> Result<(), Box<dyn std::error::Error>> {
    let ec = jerasure_rs::erasure::ErasureCodeBuilder::new()
        .coding_method(method)
        .k(NonZeroI32::new(k).unwrap())
        .m(NonZeroI32::new(m).unwrap())
        .packet_size(NonZeroI32::new(128).unwrap())
        .tech(jerasure_rs::erasure::Technique::BitMatrix)
        .build()?;
    general_test(ec)
}

fn test_sechdule(k: i32, m: i32, method: CodingMethod) -> Result<(), Box<dyn std::error::Error>> {
    let ec = jerasure_rs::erasure::ErasureCodeBuilder::new()
        .coding_method(method)
        .k(NonZeroI32::new(k).unwrap())
        .m(NonZeroI32::new(m).unwrap())
        .packet_size(NonZeroI32::new(128).unwrap())
        .tech(jerasure_rs::erasure::Technique::Schedule)
        .build()?;
    general_test(ec)?;
    if m == 2 {
        let ec = jerasure_rs::erasure::ErasureCodeBuilder::new()
            .coding_method(method)
            .k(NonZeroI32::new(k).unwrap())
            .m(NonZeroI32::new(m).unwrap())
            .packet_size(NonZeroI32::new(128).unwrap())
            .tech(jerasure_rs::erasure::Technique::ScheduleCache)
            .build()?;
        general_test(ec)?;
    }
    Ok(())
}

fn general_test(ec: jerasure_rs::erasure::ErasureCode) -> Result<(), Box<dyn std::error::Error>> {
    let k = ec.k();
    let m = ec.m();
    let source_k = make_rand_blk(k.try_into().unwrap(), BLK_SIZE);
    // # encode
    let data = source_k.clone();
    let mut encoded = make_zero_blk(m.try_into().unwrap(), BLK_SIZE);
    ec.encode(&data, &mut encoded)?;
    let encoded = encoded;
    // encode should not modify source data
    assert_eq!(data, source_k);

    // # decode
    // ## erase one data block
    let erase_idx = rand::random_range(0..k.try_into().unwrap());
    let mut erased_data = data.clone();
    let mut erased_code = encoded.clone();
    erased_data[erase_idx] = vec![0_u8; BLK_SIZE];
    ec.decode(&mut erased_data, &mut erased_code, &[erase_idx as i32])?;
    assert_eq!(erased_data, data);
    assert_eq!(erased_code, encoded);
    // ## erase one code block
    let erase_idx = rand::random_range(0..m.try_into().unwrap());
    let mut erased_data = data.clone();
    let mut erased_code = encoded.clone();
    erased_code[erase_idx] = vec![0_u8; BLK_SIZE];
    ec.decode(
        &mut erased_data,
        &mut erased_code,
        &[i32::try_from(erase_idx).unwrap() + k],
    )?;
    assert_eq!(erased_data, data);
    assert_eq!(erased_code, encoded);
    // ## erase m code blocks
    let mut erased_data = data.clone();
    let mut erased_code = encoded.clone();
    let erase_idx = {
        let mut idx: Vec<_> = (0..m).collect();
        idx.shuffle(&mut rand::rng());
        idx[0..m as usize].to_vec()
    };
    erase_idx.iter().for_each(|i| {
        if i < &k {
            erased_data[*i as usize] = vec![0_u8; BLK_SIZE];
        } else {
            erased_code[(*i - k) as usize] = vec![0_u8; BLK_SIZE];
        }
    });
    ec.decode(&mut erased_data, &mut erased_code, &erase_idx)?;
    assert_eq!(erased_data, data);
    assert_eq!(erased_code, encoded);

    Ok(())
}

fn fail_test(k: i32, m: i32, method: CodingMethod) {
    // # k <0
    let ec = jerasure_rs::erasure::ErasureCodeBuilder::new()
        .coding_method(method)
        .k(NonZeroI32::new(-1).unwrap())
        .m(NonZeroI32::new(m).unwrap())
        .tech(jerasure_rs::erasure::Technique::Matrix)
        .build();
    assert!(matches!(ec, Err(jerasure_rs::Error::InvalidArguments(_))));

    // # m <0
    let ec = jerasure_rs::erasure::ErasureCodeBuilder::new()
        .coding_method(method)
        .k(NonZeroI32::new(k).unwrap())
        .m(NonZeroI32::new(-1).unwrap())
        .tech(jerasure_rs::erasure::Technique::Matrix)
        .build();
    assert!(matches!(ec, Err(jerasure_rs::Error::InvalidArguments(_))));

    // # k + m > 255
    let ec = jerasure_rs::erasure::ErasureCodeBuilder::new()
        .coding_method(method)
        .k(NonZeroI32::new(255).unwrap())
        .m(NonZeroI32::new(2).unwrap())
        .tech(jerasure_rs::erasure::Technique::Matrix)
        .build();
    assert!(matches!(ec, Err(jerasure_rs::Error::InvalidArguments(_))));

    if !matches!(method, CodingMethod::ReedSolVand) {
        // # bitmatrix
        // packet size < 0
        let ec = jerasure_rs::erasure::ErasureCodeBuilder::new()
            .coding_method(method)
            .k(NonZeroI32::new(k).unwrap())
            .m(NonZeroI32::new(m).unwrap())
            .packet_size(NonZeroI32::new(-1).unwrap())
            .tech(jerasure_rs::erasure::Technique::BitMatrix)
            .build();
        assert!(matches!(ec, Err(jerasure_rs::Error::InvalidArguments(_))));
        // packet size not multiple of machine long size
        let ec = jerasure_rs::erasure::ErasureCodeBuilder::new()
            .coding_method(method)
            .k(NonZeroI32::new(k).unwrap())
            .m(NonZeroI32::new(m).unwrap())
            .packet_size(NonZeroI32::new(42).unwrap())
            .tech(jerasure_rs::erasure::Technique::BitMatrix)
            .build();
        assert!(matches!(ec, Err(jerasure_rs::Error::InvalidArguments(_))));

        // # schedule
        // packet size < 0
        let ec = jerasure_rs::erasure::ErasureCodeBuilder::new()
            .coding_method(method)
            .k(NonZeroI32::new(k).unwrap())
            .m(NonZeroI32::new(m).unwrap())
            .packet_size(NonZeroI32::new(-1).unwrap())
            .tech(jerasure_rs::erasure::Technique::Schedule)
            .build();
        assert!(matches!(ec, Err(jerasure_rs::Error::InvalidArguments(_))));
        // packet size not multiple of machine long size
        let ec = jerasure_rs::erasure::ErasureCodeBuilder::new()
            .coding_method(method)
            .k(NonZeroI32::new(k).unwrap())
            .m(NonZeroI32::new(m).unwrap())
            .packet_size(NonZeroI32::new(42).unwrap())
            .tech(jerasure_rs::erasure::Technique::Schedule)
            .build();
        assert!(matches!(ec, Err(jerasure_rs::Error::InvalidArguments(_))));

        // # schedule cache
        // m != 2
        let ec = jerasure_rs::erasure::ErasureCodeBuilder::new()
            .coding_method(method)
            .k(NonZeroI32::new(k).unwrap())
            .m(NonZeroI32::new(3).unwrap())
            .packet_size(NonZeroI32::new(128).unwrap())
            .tech(jerasure_rs::erasure::Technique::ScheduleCache)
            .build();
        assert!(matches!(ec, Err(jerasure_rs::Error::NotSupported(_))));
        if m == 2 {
            // packet size < 0
            let ec = jerasure_rs::erasure::ErasureCodeBuilder::new()
                .coding_method(method)
                .k(NonZeroI32::new(k).unwrap())
                .m(NonZeroI32::new(m).unwrap())
                .packet_size(NonZeroI32::new(-1).unwrap())
                .tech(jerasure_rs::erasure::Technique::ScheduleCache)
                .build();
            assert!(matches!(ec, Err(jerasure_rs::Error::InvalidArguments(_))));
            // packet size not multiple of machine long size
            let ec = jerasure_rs::erasure::ErasureCodeBuilder::new()
                .coding_method(method)
                .k(NonZeroI32::new(k).unwrap())
                .m(NonZeroI32::new(m).unwrap())
                .packet_size(NonZeroI32::new(42).unwrap())
                .tech(jerasure_rs::erasure::Technique::ScheduleCache)
                .build();
            assert!(matches!(ec, Err(jerasure_rs::Error::InvalidArguments(_))));
        }
    }

    let ec = jerasure_rs::erasure::ErasureCodeBuilder::new()
        .coding_method(method)
        .k(NonZeroI32::new(k).unwrap())
        .m(NonZeroI32::new(m).unwrap())
        .tech(jerasure_rs::erasure::Technique::Matrix)
        .build()
        .unwrap();

    // # buffer not aligned
    let data = make_rand_blk(k.try_into().unwrap(), BLK_SIZE + 1);
    let mut code = make_zero_blk(m.try_into().unwrap(), BLK_SIZE + 1);
    let res = ec.encode(&data, &mut code);
    assert!(matches!(res, Err(jerasure_rs::Error::NotAligned(_))));

    // # not enough data
    let data = make_rand_blk(k.try_into().unwrap(), BLK_SIZE);
    let mut code = make_zero_blk(m.try_into().unwrap(), BLK_SIZE);
    let data = data[0..(k - 1) as usize].to_vec();
    let res = ec.encode(&data, &mut code);
    assert!(matches!(res, Err(jerasure_rs::Error::InvalidArguments(_))));

    // # not enough code blocks
    let data = make_rand_blk(k.try_into().unwrap(), BLK_SIZE);
    let code = make_zero_blk(m.try_into().unwrap(), BLK_SIZE);
    let mut code = code[0..(m - 1) as usize].to_vec();
    let res = ec.encode(&data, &mut code);
    assert!(matches!(res, Err(jerasure_rs::Error::InvalidArguments(_))));

    // # decode with invalid erased indices
    let data = make_rand_blk(k.try_into().unwrap(), BLK_SIZE);
    let mut code = make_zero_blk(m.try_into().unwrap(), BLK_SIZE);
    ec.encode(&data, &mut code).unwrap();
    // Invalid index (negative)
    let mut erased_data = data.clone();
    let mut erased_code = code.clone();
    let res = ec.decode(&mut erased_data, &mut erased_code, &[-1]);
    assert!(matches!(res, Err(jerasure_rs::Error::InvalidArguments(_))));
    // Invalid index (out of bounds)
    let mut erased_data = data.clone();
    let mut erased_code = code.clone();
    let res = ec.decode(&mut erased_data, &mut erased_code, &[k + m]);
    assert!(matches!(res, Err(jerasure_rs::Error::InvalidArguments(_))));

    // # decode with too many erased blocks
    let mut erased_data = data.clone();
    let mut erased_code = code.clone();
    erased_data[0] = vec![0_u8; BLK_SIZE];
    erased_code[0] = vec![0_u8; BLK_SIZE];
    erased_code[1] = vec![0_u8; BLK_SIZE];
    let res = ec.decode(&mut erased_data, &mut erased_code, &[0, 4, 5]);
    assert!(
        matches!(res, Err(jerasure_rs::Error::TooManyErased(_, _))),
        "res:{:?}",
        res
    );

    // # decode with misaligned buffers
    let mut erased_data = make_rand_blk(k.try_into().unwrap(), BLK_SIZE + 1);
    let mut erased_code = make_zero_blk(m.try_into().unwrap(), BLK_SIZE + 1);
    let res = ec.decode(&mut erased_data, &mut erased_code, &[0]);
    assert!(matches!(res, Err(jerasure_rs::Error::NotAligned(_))));

    // # encode with empty data
    let data: Vec<Vec<u8>> = vec![];
    let mut code = make_zero_blk(m.try_into().unwrap(), BLK_SIZE);
    let res = ec.encode(&data, &mut code);
    assert!(matches!(res, Err(jerasure_rs::Error::InvalidArguments(_))));

    // # decode with empty data
    let data: Vec<Vec<u8>> = vec![];
    let mut code = make_zero_blk(m.try_into().unwrap(), BLK_SIZE);
    let res = ec.decode(&mut data.clone(), &mut code, &[0]);
    assert!(matches!(res, Err(jerasure_rs::Error::InvalidArguments(_))));
}
