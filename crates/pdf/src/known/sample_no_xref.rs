use std::collections::BTreeMap;

use crate::{
    array, dict,
    dictionary::Dictionary,
    dictionary_struct, int, name,
    object::{Object, StringFormat},
    real, reference,
    stream::Stream,
    string_lit,
    xref::{Xref, XrefEntry},
    xref_n, ObjectId,
};

pub fn sample_no_xref_pdf_trailer<'a>() -> Dictionary<'a> {
    let mut dictionary = Dictionary::new();
    dictionary.insert(b"Size".to_vec(), int!(12));
    dictionary.insert(b"Root".to_vec(), reference!(1, 0));
    dictionary.insert(b"Info".to_vec(), reference!(10, 0));
    dictionary
}

pub fn sample_no_xref_pdf_objects<'a>() -> BTreeMap<ObjectId, Object<'a>> {
    let object_1_v_0 = dict!(dictionary_struct! {
        "Type" => name!("Catalog"),
        "Outlines" => reference!(2, 0),
        "Pages" => reference!(3, 0),
    });

    let object_2_v_0 = dict!(dictionary_struct! {
        "Type" => name!("Outlines"),
        "Count" => int!(0),
    });

    let object_3_v_0 = dict!(dictionary_struct! {
        "Type" => name!("Pages"),
        "Count" => int!(2),
        "Kids" => array!(reference!(4, 0), reference!(6, 0))
    });

    let object_4_v_0 = dict!(dictionary_struct! {
        "Type" => name!("Page"),
        "Parent" => reference!(3, 0),
        "Resources" => dict!(dictionary_struct! {
            "Font" => dict!(dictionary_struct! {
                "F1" => reference!(9, 0)
            }),
            "ProcSet" => reference!(8, 0),
        }),
        "MediaBox" => array!(
            int!(0),
            int!(0),
            real!(612.0000),
            real!(792.0000),
        ),
        "Contents" => reference!(5, 0),
    });

    let object_5_v_0 = Object::Stream(Stream {
        dict: dictionary_struct! {
            "Length" => int!(1074),
        },
        content: &[
            50, 32, 74, 13, 10, 66, 84, 13, 10, 48, 32, 48, 32, 48, 32, 114, 103, 13, 10, 47, 70,
            49, 32, 48, 48, 50, 55, 32, 84, 102, 13, 10, 53, 55, 46, 51, 55, 53, 48, 32, 55, 50,
            50, 46, 50, 56, 48, 48, 32, 84, 100, 13, 10, 40, 32, 65, 32, 83, 105, 109, 112, 108,
            101, 32, 80, 68, 70, 32, 70, 105, 108, 101, 32, 41, 32, 84, 106, 13, 10, 69, 84, 13,
            10, 66, 84, 13, 10, 47, 70, 49, 32, 48, 48, 49, 48, 32, 84, 102, 13, 10, 54, 57, 46,
            50, 53, 48, 48, 32, 54, 56, 56, 46, 54, 48, 56, 48, 32, 84, 100, 13, 10, 40, 32, 84,
            104, 105, 115, 32, 105, 115, 32, 97, 32, 115, 109, 97, 108, 108, 32, 100, 101, 109,
            111, 110, 115, 116, 114, 97, 116, 105, 111, 110, 32, 46, 112, 100, 102, 32, 102, 105,
            108, 101, 32, 45, 32, 41, 32, 84, 106, 13, 10, 69, 84, 13, 10, 66, 84, 13, 10, 47, 70,
            49, 32, 48, 48, 49, 48, 32, 84, 102, 13, 10, 54, 57, 46, 50, 53, 48, 48, 32, 54, 54,
            52, 46, 55, 48, 52, 48, 32, 84, 100, 13, 10, 40, 32, 106, 117, 115, 116, 32, 102, 111,
            114, 32, 117, 115, 101, 32, 105, 110, 32, 116, 104, 101, 32, 86, 105, 114, 116, 117,
            97, 108, 32, 77, 101, 99, 104, 97, 110, 105, 99, 115, 32, 116, 117, 116, 111, 114, 105,
            97, 108, 115, 46, 32, 77, 111, 114, 101, 32, 116, 101, 120, 116, 46, 32, 65, 110, 100,
            32, 109, 111, 114, 101, 32, 41, 32, 84, 106, 13, 10, 69, 84, 13, 10, 66, 84, 13, 10,
            47, 70, 49, 32, 48, 48, 49, 48, 32, 84, 102, 13, 10, 54, 57, 46, 50, 53, 48, 48, 32,
            54, 53, 50, 46, 55, 53, 50, 48, 32, 84, 100, 13, 10, 40, 32, 116, 101, 120, 116, 46,
            32, 65, 110, 100, 32, 109, 111, 114, 101, 32, 116, 101, 120, 116, 46, 32, 65, 110, 100,
            32, 109, 111, 114, 101, 32, 116, 101, 120, 116, 46, 32, 65, 110, 100, 32, 109, 111,
            114, 101, 32, 116, 101, 120, 116, 46, 32, 41, 32, 84, 106, 13, 10, 69, 84, 13, 10, 66,
            84, 13, 10, 47, 70, 49, 32, 48, 48, 49, 48, 32, 84, 102, 13, 10, 54, 57, 46, 50, 53,
            48, 48, 32, 54, 50, 56, 46, 56, 52, 56, 48, 32, 84, 100, 13, 10, 40, 32, 65, 110, 100,
            32, 109, 111, 114, 101, 32, 116, 101, 120, 116, 46, 32, 65, 110, 100, 32, 109, 111,
            114, 101, 32, 116, 101, 120, 116, 46, 32, 65, 110, 100, 32, 109, 111, 114, 101, 32,
            116, 101, 120, 116, 46, 32, 65, 110, 100, 32, 109, 111, 114, 101, 32, 116, 101, 120,
            116, 46, 32, 65, 110, 100, 32, 109, 111, 114, 101, 32, 41, 32, 84, 106, 13, 10, 69, 84,
            13, 10, 66, 84, 13, 10, 47, 70, 49, 32, 48, 48, 49, 48, 32, 84, 102, 13, 10, 54, 57,
            46, 50, 53, 48, 48, 32, 54, 49, 54, 46, 56, 57, 54, 48, 32, 84, 100, 13, 10, 40, 32,
            116, 101, 120, 116, 46, 32, 65, 110, 100, 32, 109, 111, 114, 101, 32, 116, 101, 120,
            116, 46, 32, 66, 111, 114, 105, 110, 103, 44, 32, 122, 122, 122, 122, 122, 46, 32, 65,
            110, 100, 32, 109, 111, 114, 101, 32, 116, 101, 120, 116, 46, 32, 65, 110, 100, 32,
            109, 111, 114, 101, 32, 116, 101, 120, 116, 46, 32, 65, 110, 100, 32, 41, 32, 84, 106,
            13, 10, 69, 84, 13, 10, 66, 84, 13, 10, 47, 70, 49, 32, 48, 48, 49, 48, 32, 84, 102,
            13, 10, 54, 57, 46, 50, 53, 48, 48, 32, 54, 48, 52, 46, 57, 52, 52, 48, 32, 84, 100,
            13, 10, 40, 32, 109, 111, 114, 101, 32, 116, 101, 120, 116, 46, 32, 65, 110, 100, 32,
            109, 111, 114, 101, 32, 116, 101, 120, 116, 46, 32, 65, 110, 100, 32, 109, 111, 114,
            101, 32, 116, 101, 120, 116, 46, 32, 65, 110, 100, 32, 109, 111, 114, 101, 32, 116,
            101, 120, 116, 46, 32, 65, 110, 100, 32, 109, 111, 114, 101, 32, 116, 101, 120, 116,
            46, 32, 41, 32, 84, 106, 13, 10, 69, 84, 13, 10, 66, 84, 13, 10, 47, 70, 49, 32, 48,
            48, 49, 48, 32, 84, 102, 13, 10, 54, 57, 46, 50, 53, 48, 48, 32, 53, 57, 50, 46, 57,
            57, 50, 48, 32, 84, 100, 13, 10, 40, 32, 65, 110, 100, 32, 109, 111, 114, 101, 32, 116,
            101, 120, 116, 46, 32, 65, 110, 100, 32, 109, 111, 114, 101, 32, 116, 101, 120, 116,
            46, 32, 41, 32, 84, 106, 13, 10, 69, 84, 13, 10, 66, 84, 13, 10, 47, 70, 49, 32, 48,
            48, 49, 48, 32, 84, 102, 13, 10, 54, 57, 46, 50, 53, 48, 48, 32, 53, 54, 57, 46, 48,
            56, 56, 48, 32, 84, 100, 13, 10, 40, 32, 65, 110, 100, 32, 109, 111, 114, 101, 32, 116,
            101, 120, 116, 46, 32, 65, 110, 100, 32, 109, 111, 114, 101, 32, 116, 101, 120, 116,
            46, 32, 65, 110, 100, 32, 109, 111, 114, 101, 32, 116, 101, 120, 116, 46, 32, 65, 110,
            100, 32, 109, 111, 114, 101, 32, 116, 101, 120, 116, 46, 32, 65, 110, 100, 32, 109,
            111, 114, 101, 32, 41, 32, 84, 106, 13, 10, 69, 84, 13, 10, 66, 84, 13, 10, 47, 70, 49,
            32, 48, 48, 49, 48, 32, 84, 102, 13, 10, 54, 57, 46, 50, 53, 48, 48, 32, 53, 53, 55,
            46, 49, 51, 54, 48, 32, 84, 100, 13, 10, 40, 32, 116, 101, 120, 116, 46, 32, 65, 110,
            100, 32, 109, 111, 114, 101, 32, 116, 101, 120, 116, 46, 32, 65, 110, 100, 32, 109,
            111, 114, 101, 32, 116, 101, 120, 116, 46, 32, 69, 118, 101, 110, 32, 109, 111, 114,
            101, 46, 32, 67, 111, 110, 116, 105, 110, 117, 101, 100, 32, 111, 110, 32, 112, 97,
            103, 101, 32, 50, 32, 46, 46, 46, 41, 32, 84, 106, 13, 10, 69, 84, 13, 10,
        ],
        allows_compression: false,
        start_position: None,
    });

    let object_6_v_0 = dict!(dictionary_struct! {
        "Type" => name!("Page"),
        "Parent" => reference!(3, 0),
        "Resources" => dict!(dictionary_struct! {
            "Font" => dict!(dictionary_struct! {
                "F1" => reference!(9, 0)
            }),
            "ProcSet" => reference!(8, 0),
        }),
        "MediaBox" => array!(
            int!(0),
            int!(0),
            real!(612.0000),
            real!(792.0000),
        ),
        "Contents" => array!(reference!(7, 0), reference!(11, 0)),
    });

    let object_7_v_0 = Object::Stream(Stream {
        dict: dictionary_struct! {
            "Length" => int!(676),
        },
        content: &[
            50, 32, 74, 13, 10, 66, 84, 13, 10, 48, 32, 48, 32, 48, 32, 114, 103, 13, 10, 47, 70,
            49, 32, 48, 48, 50, 55, 32, 84, 102, 13, 10, 53, 55, 46, 51, 55, 53, 48, 32, 55, 50,
            50, 46, 50, 56, 48, 48, 32, 84, 100, 13, 10, 40, 32, 83, 105, 109, 112, 108, 101, 32,
            80, 68, 70, 32, 70, 105, 108, 101, 32, 50, 32, 41, 32, 84, 106, 13, 10, 69, 84, 13, 10,
            66, 84, 13, 10, 47, 70, 49, 32, 48, 48, 49, 48, 32, 84, 102, 13, 10, 54, 57, 46, 50,
            53, 48, 48, 32, 54, 56, 56, 46, 54, 48, 56, 48, 32, 84, 100, 13, 10, 40, 32, 46, 46,
            46, 99, 111, 110, 116, 105, 110, 117, 101, 100, 32, 102, 114, 111, 109, 32, 112, 97,
            103, 101, 32, 49, 46, 32, 89, 101, 116, 32, 109, 111, 114, 101, 32, 116, 101, 120, 116,
            46, 32, 65, 110, 100, 32, 109, 111, 114, 101, 32, 116, 101, 120, 116, 46, 32, 65, 110,
            100, 32, 109, 111, 114, 101, 32, 116, 101, 120, 116, 46, 32, 41, 32, 84, 106, 13, 10,
            69, 84, 13, 10, 66, 84, 13, 10, 47, 70, 49, 32, 48, 48, 49, 48, 32, 84, 102, 13, 10,
            54, 57, 46, 50, 53, 48, 48, 32, 54, 55, 54, 46, 54, 53, 54, 48, 32, 84, 100, 13, 10,
            40, 32, 65, 110, 100, 32, 109, 111, 114, 101, 32, 116, 101, 120, 116, 46, 32, 65, 110,
            100, 32, 109, 111, 114, 101, 32, 116, 101, 120, 116, 46, 32, 65, 110, 100, 32, 109,
            111, 114, 101, 32, 116, 101, 120, 116, 46, 32, 65, 110, 100, 32, 109, 111, 114, 101,
            32, 116, 101, 120, 116, 46, 32, 65, 110, 100, 32, 109, 111, 114, 101, 32, 41, 32, 84,
            106, 13, 10, 69, 84, 13, 10, 66, 84, 13, 10, 47, 70, 49, 32, 48, 48, 49, 48, 32, 84,
            102, 13, 10, 54, 57, 46, 50, 53, 48, 48, 32, 54, 54, 52, 46, 55, 48, 52, 48, 32, 84,
            100, 13, 10, 40, 32, 116, 101, 120, 116, 46, 32, 79, 104, 44, 32, 104, 111, 119, 32,
            98, 111, 114, 105, 110, 103, 32, 116, 121, 112, 105, 110, 103, 32, 116, 104, 105, 115,
            32, 115, 116, 117, 102, 102, 46, 32, 66, 117, 116, 32, 110, 111, 116, 32, 97, 115, 32,
            98, 111, 114, 105, 110, 103, 32, 97, 115, 32, 119, 97, 116, 99, 104, 105, 110, 103, 32,
            41, 32, 84, 106, 13, 10, 69, 84, 13, 10, 66, 84, 13, 10, 47, 70, 49, 32, 48, 48, 49,
            48, 32, 84, 102, 13, 10, 54, 57, 46, 50, 53, 48, 48, 32, 54, 53, 50, 46, 55, 53, 50,
            48, 32, 84, 100, 13, 10, 40, 32, 112, 97, 105, 110, 116, 32, 100, 114, 121, 46, 32, 65,
            110, 100, 32, 109, 111, 114, 101, 32, 116, 101, 120, 116, 46, 32, 65, 110, 100, 32,
            109, 111, 114, 101, 32, 116, 101, 120, 116, 46, 32, 65, 110, 100, 32, 109, 111, 114,
            101, 32, 116, 101, 120, 116, 46, 32, 65, 110, 100, 32, 109, 111, 114, 101, 32, 116,
            101, 120, 116, 46, 32, 41, 32, 84, 106, 13, 10, 69, 84, 13, 10, 66, 84, 13, 10, 47, 70,
            49, 32, 48, 48, 49, 48, 32, 84, 102, 13, 10, 54, 57, 46, 50, 53, 48, 48, 32, 54, 52,
            48, 46, 56, 48, 48, 48, 32, 84, 100, 13, 10, 40, 32, 66, 111, 114, 105, 110, 103, 46,
            32, 32, 77, 111, 114, 101, 44, 32, 97, 32, 108, 105, 116, 116, 108, 101, 32, 109, 111,
            114, 101, 32, 116, 101, 120, 116, 46, 32, 84, 104, 101, 32, 101, 110, 100, 44, 32, 97,
            110, 100, 32, 106, 117, 115, 116, 32, 97, 115, 32, 119, 101, 108, 108, 46, 32, 41, 32,
            84, 106, 13, 10, 69, 84, 13, 10,
        ],
        allows_compression: false,
        start_position: None,
    });

    let object_8_v_0 = Object::Array(vec![name!("PDF"), name!("Text")]);

    let object_9_v_0 = Object::Dictionary(dictionary_struct! {
        "Type" => name!("Font"),
        "Subtype" => name!("Type1"),
        "Name" => name!("F1"),
        "BaseFont" => name!("Helvetica"),
        "Encoding" => name!("WinAnsiEncoding"),
    });

    let object_10_v_0 = Object::Dictionary(dictionary_struct! {
        "Creator" => string_lit!(b"Rave (http://www.nevrona.com/rave)"),
        "Producer" => string_lit!(b"Nevrona Designs"),
        "CreationDate" => string_lit!(b"D:20060301072826"),
    });

    let object_11_v_0 = Object::Stream(Stream {
        dict: dictionary_struct! {
            "Length" => int!(19)
        },
        content: &[
            53, 49, 48, 32, 53, 48, 48, 32, 109, 13, 10, 54, 49, 48, 32, 54, 48, 48, 32, 108, 13,
            10, 37, 32, 76, 105, 110, 101, 58, 13, 10, 53, 48, 48, 32, 53, 48, 48, 32, 109, 13, 10,
            54, 48, 48, 32, 54, 48, 48, 32, 108, 13, 10, 37, 32, 90, 101, 114, 111, 32, 108, 101,
            110, 103, 116, 104, 32, 108, 105, 110, 101, 58, 13, 10, 49, 48, 32, 49, 48, 32, 109,
            13, 10, 49, 48, 32, 49, 48, 32, 108, 13, 10, 102, 13, 10,
        ],
        allows_compression: false,
        start_position: None,
    });

    btreemap! {
        (1, 0) => object_1_v_0,
        (2, 0) => object_2_v_0,
        (3, 0) => object_3_v_0,
        (4, 0) => object_4_v_0,
        (5, 0) => object_5_v_0,
        (6, 0) => object_6_v_0,
        (7, 0) => object_7_v_0,
        (8, 0) => object_8_v_0,
        (9, 0) => object_9_v_0,
        (10, 0) => object_10_v_0,
        (11, 0) => object_11_v_0,
    }
}
