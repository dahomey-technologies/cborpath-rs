use crate::{
    tests::util::{cbor_to_diag, diag_to_cbor, log_try_init},
    CborPath,
};
use cbor_data::{Cbor, CborBuilder, CborOwned, ItemKind, Writer};
use std::borrow::Cow;

/// Based on https://redis.io/commands/json.clear/
fn clear(cbor_path: &CborPath, cbor: &Cbor) -> (CborOwned, usize) {
    let mut num_cleared_values = 0;
    let new_value = cbor_path.write(cbor, |old_value| {
        let new_value = match old_value.kind() {
            ItemKind::Pos(_) | ItemKind::Neg(_) => {
                num_cleared_values += 1;
                CborBuilder::new().write_pos(0, None)
            }
            ItemKind::Float(_) => {
                num_cleared_values += 1;
                CborBuilder::new().write_lit(cbor_data::Literal::L2(0), None)
            }
            ItemKind::Str(_)
            | ItemKind::Bytes(_)
            | ItemKind::Bool(_)
            | ItemKind::Null
            | ItemKind::Undefined
            | ItemKind::Simple(_) => CborBuilder::new().write_item(old_value),
            ItemKind::Array(_) => {
                num_cleared_values += 1;
                CborBuilder::new().write_array(None, |_| ())
            }
            ItemKind::Dict(_) => {
                num_cleared_values += 1;
                CborBuilder::new().write_dict(None, |_| ())
            }
        };

        log::trace!("old_value:{old_value}, new_value:{new_value}");
        Some(Cow::Owned(new_value))
    });

    (new_value, num_cleared_values)
}

#[test]
fn clear_values() {
    log_try_init();

    let cbor = diag_to_cbor(
        r#"{"obj":{"a":1, "b":2}, "arr":[1,2,3], "str": "foo", "bool": true, "int": 42, "float": 3.14}"#,
    );

    let cbor_path = CborPath::builder().wildcard().build();
    let (new_value, num_cleared_values) = clear(&cbor_path, &cbor);

    log::trace!("new_value:{new_value:?}");
    log::trace!("new_value:{}", cbor_to_diag(&new_value));

    assert_eq!(
        r#"{"obj":{},"arr":[],"str":"foo","bool":true,"int":0,"float":0.0_1}"#,
        cbor_to_diag(&new_value)
    );
    assert_eq!(4, num_cleared_values);
}
